use axum::{extract::{ws::{Message, WebSocket}, WebSocketUpgrade}, response::Response};
use common::websocket::{ToClient, ToServer};
use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use tokio::{select, sync::{broadcast, mpsc::{self, unbounded_channel, UnboundedReceiver}}};

pub struct Client {
  id: u64,
  socket: SplitSink<WebSocket, Message>
}

impl Client {
  pub fn get_id(&self) -> u64 {
    self.id
  }

  pub async fn send(&mut self, message: ToClient) {
    let _ = self.socket.send(Message::Binary(serde_cbor::to_vec(&message).unwrap())).await;
  }
}

pub trait SocketHandler {
  fn on_connect(&mut self, client: Client) -> impl std::future::Future<Output = ()> + std::marker::Send;
  fn on_message(&mut self, client_id: u64, message: ToServer) -> impl std::future::Future<Output = ()> + std::marker::Send;
  fn on_disconnect(&mut self, client_id: u64) -> impl std::future::Future<Output = ()> + std::marker::Send;
}

pub struct SocketEndpoint {
  message_sender: mpsc::UnboundedSender<ServerMessage>,
  kill_sender: broadcast::Sender<()>
}

impl SocketEndpoint {
  pub fn new(socket_handler: impl SocketHandler + Send + 'static) -> Self  {
    let (message_sender, message_receiver) = unbounded_channel();
    let (kill_sender, _) = broadcast::channel(1);
    let state = SocketEndpoint {
      message_sender, kill_sender,
    };
    tokio::spawn(pass_messages(message_receiver, socket_handler));
    state
  }

  pub fn handler(&self, ws: WebSocketUpgrade) -> Response {
    let message_sender = self.message_sender.clone();
    let kill_receiver = self.kill_sender.subscribe();
    ws.on_upgrade(move |socket| on_upgrade(socket, message_sender, kill_receiver))
  }
}

impl Drop for SocketEndpoint {
  fn drop(&mut self) {
    self.kill_sender.send(()).unwrap();
  }
}

enum ServerMessage {
  NewClient(Client),
  Message {client_id: u64, message: ToServer },
  Disconnect {client_id: u64},
}

async fn on_upgrade(socket: WebSocket, message_sender: mpsc::UnboundedSender<ServerMessage>, mut kill_receiver: broadcast::Receiver<()>) {
  let id = rand::random::<u64>();
  let (to_client, mut from_client) = socket.split();
  let client = Client { id, socket: to_client };
  message_sender.send(ServerMessage::NewClient(client)).unwrap();
  loop {
    select! {
      Some(Ok(message)) = from_client.next() => {
        match message {
          Message::Binary(message) => {
            let Ok(message) = serde_cbor::from_slice(&message) else { break; };
            message_sender.send(ServerMessage::Message { client_id: id, message }).unwrap();
          },
          Message::Close(_) => break,
          _ => continue
        }
      },
      _ = kill_receiver.recv() => {
        break;
      },
      else => {
        break;
      }
    }
  }
  message_sender.send(ServerMessage::Disconnect {client_id: id}).unwrap();
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