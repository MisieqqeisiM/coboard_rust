mod socket_endpoint;

use std::collections::HashMap;
use axum::Router;
use common::{entities::Position, websocket::{ToClient, ToServer}};
use socket_endpoint::{socket_endpoint, Client, SocketHandler};
use tokio::net::TcpListener;

struct Handler {
    clients: HashMap<u64, Client>,
    positions: HashMap<u64, Position>,
}

impl Handler {
    fn new() -> Self{
        Self {
            clients: HashMap::new(),
            positions: HashMap::new(),
        }
    }

    async fn broadcast(&mut self, message: ToClient) {
        for client in self.clients.values_mut() {
            client.send(message.clone()).await;
        }
    }
}

impl SocketHandler for Handler {
    async fn on_connect(&mut self, mut client: Client) {
        let id = client.id;
        client.send(ToClient::ClientList { 
            clients: self.positions.iter()
                .map(|(id, pos)| (id.to_owned(), pos.to_owned()))
                .collect()
        }).await;
        self.clients.insert(id, client);
        self.broadcast(ToClient::NewClient { id }).await;
    }

    async fn on_message(&mut self, client_id: u64, message: ToServer) {
        match message {
            ToServer::Move { x, y } => {
                self.positions.insert(client_id, Position { x, y } );
                self.broadcast(ToClient::ClientMoved { id: client_id, x, y } ).await;
            }
        };
    }
    
    async fn on_disconnect(&mut self, client_id: u64) {
        self.clients.remove(&client_id);
        self.positions.remove(&client_id);
        self.broadcast(ToClient::ClientDisconnected { id: client_id } ).await;
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    let app = Router::new().nest("/ws", socket_endpoint(Handler::new()));
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    tracing::info!("starting server");
    axum::serve(listener, app).await.unwrap();
}
