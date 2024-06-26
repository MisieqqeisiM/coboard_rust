use std::{collections::HashMap, sync::Arc};

use axum::{extract::{Path, State, WebSocketUpgrade}, http::StatusCode, response::{IntoResponse, Response}, routing::get, Router};
use tokio::sync::Mutex;
use tracing::info;
use uuid::Uuid;

use crate::{board::Board, socket_endpoint::SocketEndpoint};

async fn ws(ws: WebSocketUpgrade, Path(socket_id): Path<Uuid>, State(state): State<Arc<Mutex<ServerState>>>) -> Response{
  let state = state.lock().await;
  match state.endpoints.get(&socket_id) {
    Some(endpoint) => endpoint.handler(ws),
    None => (StatusCode::NOT_FOUND, "Socket does not exist").into_response(),
  }
} 

async fn create_board(State(state): State<Arc<Mutex<ServerState>>>) -> Response {
  let mut state = state.lock().await; 
  let uuid = Uuid::new_v4();
  info!("{}", uuid);
  state.endpoints.insert(uuid, SocketEndpoint::new(Board::new()));
  format!("/boards/{}", uuid.to_string()).into_response()
}

struct ServerState {
  endpoints: HashMap<Uuid, SocketEndpoint>,
}

pub fn board_server() -> Router {
  let mut state = ServerState {
    endpoints: HashMap::new(),
  };
  state.endpoints.insert(Uuid::new_v4(), SocketEndpoint::new(Board::new()));
  Router::new().route("/boards/:socket_id", get(ws))
    .route("/create_board", get(create_board))
    .with_state(Arc::new(Mutex::new(state)))
}