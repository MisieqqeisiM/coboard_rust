use std::{collections::HashMap, sync::Arc};

use axum::{extract::{Path, Query, State, WebSocketUpgrade}, http::StatusCode, response::{IntoResponse, Response}, routing::{get, post}, Router};
use serde::Deserialize;
use tokio::sync::Mutex;
use tracing::info;

use crate::{board::Board, socket_endpoint::SocketEndpoint};

const MAIN_SERVER_URL: &'static str = "http://localhost:8080/internal";

async fn ws(ws: WebSocketUpgrade, Path(socket_id): Path<String>, State(state): State<Arc<Mutex<ServerState>>>) -> Response{
  let state = state.lock().await;
  match state.endpoints.get(&socket_id) {
    Some(endpoint) => endpoint.handler(ws),
    None => (StatusCode::NOT_FOUND, "Socket does not exist").into_response(),
  }
} 

async fn delete_board(state: Arc<Mutex<ServerState>>, name: String) {
  let client = reqwest::Client::new();
  client.delete(format!("{MAIN_SERVER_URL}/delete_board"))
    .query(&[("name", &name)])
    .send().await.unwrap();
  let mut state = state.lock().await; 
  state.endpoints.remove(&name);
  info!("Board unloaded: {name}");
}

#[derive(Deserialize)]
struct CreateBoardPars {
  name: String
}

async fn create_board(Query(CreateBoardPars { name }): Query<CreateBoardPars>, State(state_arc): State<Arc<Mutex<ServerState>>>) -> Response {
  let mut state = state_arc.lock().await; 
  let state_arc = state_arc.clone();
  let name_clone = name.clone();
  state.endpoints.insert(name.clone(), SocketEndpoint::new(Board::new(move || delete_board(state_arc, name_clone))));
  info!("Board loaded: {name}");
  format!("/boards/{name}").into_response()
}

struct ServerState {
  endpoints: HashMap<String, SocketEndpoint>,
}

pub fn board_server() -> Router {
  let state = ServerState {
    endpoints: HashMap::new(),
  };
  Router::new().route("/boards/:socket_id", get(ws))
    .route("/create_board", post(create_board))
    .with_state(Arc::new(Mutex::new(state)))
}