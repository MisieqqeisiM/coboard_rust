use std::collections::HashMap;

use common::{entities::Position, websocket::{ToClient, ToServer}};

use crate::socket_endpoint::{Client, SocketHandler};

pub struct Board {
  clients: HashMap<u64, Client>,
  positions: HashMap<u64, Position>,
}

impl Board {
  pub fn new() -> Self{
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
}