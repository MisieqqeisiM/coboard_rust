use std::{collections::HashMap, sync::Arc};

use axum::{extract::{ws::{Message, WebSocket}, State, WebSocketUpgrade}, response::Response, routing::get, Router};
use common::{entities::Position, websocket::{ToClient, ToServer}};
use futures_util::{select, FutureExt, SinkExt, StreamExt};
use tokio::{net::TcpListener, sync::{broadcast::{self, error::SendError, Sender}, Mutex}};

async fn handler(ws: WebSocketUpgrade, State(app): State<AppState>) -> Response {
    ws.on_upgrade(move|socket| on_upgrade(socket, app))
}

async fn on_upgrade(socket: WebSocket, app: AppState) {
    let id = rand::random::<u64>();
    let client = Client { id };
    handle_socket(socket, client, &app).await;
    app.disconnect_client(id).await;
}

struct Client {
    id: u64
}

impl Client {
    async fn handle_message(&self, message: ToServer, app: &AppState) {
        match message {
            ToServer::Move { x, y } => {
                app.move_client(self.id, Position {x, y}).await;
            }
        };
    }
}


async fn handle_socket(socket: WebSocket, client: Client, app: &AppState) -> Option<()>{ 
    let (mut tx, mut from_client) = socket.split();
    tracing::info!("new websocket connection");
    tx.send(Message::Binary((serde_cbor::to_vec(&ToClient::ClientList { clients: app.get_clients().await })).unwrap())).await.ok()?;
    let mut to_client = app.new_client(client.id).await;
    loop {
        select! {
            message = to_client.recv().fuse() => {
                if let Ok(message) = message {
                    tx.send(Message::Binary(serde_cbor::to_vec(&message).unwrap())).await.ok()?;
                }
            },
            message = from_client.next().fuse() => {
                let message = message?.ok()?;
                if let Message::Binary(data) = message {
                    let message: ToServer = serde_cbor::from_slice(&data).ok()?;
                    tracing::info!("message received: {:?}", message);
                    client.handle_message(message, &app).await;
                }
            },
        }
    }
}

#[derive(Clone)]
struct AppState {
    clients: Arc<Mutex<HashMap<u64, Position>>>,
    broadcast: Arc<Mutex<Sender<ToClient>>>
}

impl AppState {
    async fn get_clients(&self) -> Vec<(u64, Position)> {
        let clients = self.clients.lock().await;
        clients.iter()
            .map(|(id, position)| (id.to_owned(), position.to_owned()))
            .collect()
    }

    async fn disconnect_client(&self, id: u64) {
        self.clients.lock().await.remove(&id);
        self.to_all(ToClient::ClientDisconnected { id }).await.unwrap();
    }

    async fn move_client(&self, id: u64, position: Position) {
        self.clients.lock().await.insert(id, position.clone());
        self.to_all(ToClient::ClientMoved { id,  x: position.x, y: position.y }).await.unwrap();
    }

    async fn new_client(&self, id: u64) -> tokio::sync::broadcast::Receiver<ToClient> {
        self.clients.lock().await.insert(id, Position { x: 0.0, y: 0.0 });
        let broadcast = self.broadcast.lock().await;
        let result = broadcast.subscribe();
        broadcast.send(ToClient::NewClient { id }).unwrap();
        result
    }

    async fn to_all(&self, msg: ToClient) -> Result<(), SendError<ToClient>> {
        self.broadcast.lock().await.send(msg)?;
        Ok(())
    }
}

fn socket_endpoint() -> Router {
    let (tx, _) = broadcast::channel(128);
    let state = AppState {
        clients: Arc::new(Mutex::new(HashMap::new())),
        broadcast: Arc::new(Mutex::new(tx))
    };
    Router::new().route("/", get(handler)).with_state(state)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    let app = Router::new().nest("/ws", socket_endpoint());
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    tracing::info!("starting server");
    axum::serve(listener, app).await.unwrap();
}
