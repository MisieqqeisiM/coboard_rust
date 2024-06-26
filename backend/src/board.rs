use std::{collections::HashMap, pin::Pin};

use common::{entities::Position, websocket::{ToClient, ToServer}};
use futures_util::Future;
use tracing::info;

use crate::socket_endpoint::{Client, SocketHandler};

type AsyncFnOnce = Box<dyn (FnOnce() -> Pin<Box<dyn Future<Output = ()> + Send>>) + Send>;

pub struct Board {
  clients: HashMap<u64, Client>,

  positions: HashMap<u64, Position>,
  delete: Option<AsyncFnOnce>,
}

impl Board {
  pub fn new<Fu>(delete: impl (FnOnce() -> Fu) + Send + 'static) -> Self
  where Fu: Future<Output=()> + Send + 'static {
    Self {
      clients: HashMap::new(),
      positions: HashMap::new(),
      delete: Some(Box::new(move || Box::pin(delete())))
    }
  }

  async fn broadcast(&mut self, message: ToClient) {
    for client in self.clients.values_mut() {
      client.send(message.clone()).await;
    }
  }
}

impl SocketHandler for Board {
  async fn on_connect(&mut self, mut client: Client) {
    let id = client.get_id();
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
  
  async fn tick(&mut self) {
    if self.clients.len() == 0 {
      if let Some(f) = self.delete.take() {
        f().await;
      }
    }
  }
}