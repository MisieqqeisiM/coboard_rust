use axum::{extract::{ws::{Message, WebSocket}, State, WebSocketUpgrade}, response::Response, routing::get, Router};
use common::websocket::{ToClient, ToServer};
use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

async fn handler(ws: WebSocketUpgrade, State(app): State<AppState>) -> Response {
  ws.on_upgrade(move|socket| on_upgrade(socket, app))
}

async fn on_upgrade(socket: WebSocket, app: AppState) {
  let id = rand::random::<u64>();
  let (to_client, mut from_client) = socket.split();
  let client = Client { id, socket: to_client };
  app.message_queue.send(ServerMessage::NewClient(client)).unwrap();
  while let Some(Ok(message)) = from_client.next().await {
    if let Message::Binary(message) = message {
      let Ok(message) = serde_cbor::from_slice(&message) else { break; };
      app.message_queue.send(ServerMessage::Message { client_id: id, message }).unwrap();
    } else if let Message::Close(_) = message {
      break;
    }
  }
  app.message_queue.send(ServerMessage::Disconnect {client_id: id}).unwrap();

}

enum ServerMessage {
  NewClient(Client),
  Message {client_id: u64, message: ToServer },
  Disconnect {client_id: u64},
}

pub struct Client {
  pub id: u64,
  socket: SplitSink<WebSocket, Message>
}

impl Client {
  pub async fn send(&mut self, message: ToClient) {
    self.socket.send(Message::Binary(serde_cbor::to_vec(&message).unwrap())).await.unwrap(); 
  }
}

pub trait SocketHandler {
  fn on_connect(&mut self, client: Client) -> impl std::future::Future<Output = ()> + std::marker::Send;
  fn on_message(&mut self, client_id: u64, message: ToServer) -> impl std::future::Future<Output = ()> + std::marker::Send;
  fn on_disconnect(&mut self, client_id: u64) -> impl std::future::Future<Output = ()> + std::marker::Send;
}

#[derive(Clone)]
struct AppState {
  message_queue: UnboundedSender<ServerMessage>
}

async fn pass_messages(
  mut channel: UnboundedReceiver<ServerMessage>, 
  mut socket_handler: impl SocketHandler,
) {
  while let Some(message) = channel.recv().await {
    match message {
      ServerMessage::NewClient(client) => socket_handler.on_connect(client).await,
      ServerMessage::Message { client_id, message } => 
        socket_handler.on_message(client_id, message).await,
      ServerMessage::Disconnect { client_id } => socket_handler.on_disconnect(client_id).await,
    };
  }
}

pub fn socket_endpoint(socket_handler: impl SocketHandler + Send + 'static) -> Router {
  let (sender, receiver) = unbounded_channel();
  let state = AppState {
    message_queue: sender,
  };
  tokio::spawn(pass_messages(receiver, socket_handler));
  Router::new().route("/", get(handler)).with_state(state)
}
