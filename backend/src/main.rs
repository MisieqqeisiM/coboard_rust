mod socket_endpoint;
mod board_server;
mod board;

use std::{collections::HashMap, sync::Arc};

use axum::{extract::{Query, State}, response::{IntoResponse, Response}, routing::{get, delete}, Router};
use board_server::board_server;
use serde::Deserialize;
use tokio::{net::TcpListener, sync::Mutex};

const INNER_BOARD_SERVER_URL: &'static str = "http://localhost:8080/board_server";
const OUTER_BOARD_SERVER_URL: &'static str = "/api/board_server";

#[derive(Deserialize)]
struct BoardUrlPars {
    name: String,
}

async fn board_url(Query(BoardUrlPars {name}): Query<BoardUrlPars>, State(state): State<Arc<Mutex<AppState>>>) -> Response {
    let mut state = state.lock().await;
    match state.board_urls.get(&name) {
        Some(url) => url.to_owned().into_response(),
        None => {
            let client = reqwest::Client::new();
            let path = client.post(format!("{INNER_BOARD_SERVER_URL}/create_board"))
                .query(&[("name", &name)])
                .send().await.unwrap().text().await.unwrap();
            let path = format!("{OUTER_BOARD_SERVER_URL}{path}");
            state.board_urls.insert(name, path.clone());
            path.into_response()
        }
    }
}

#[derive(Deserialize)]
struct DeleteBoardPars {
    name: String
}

async fn delete_board(Query(DeleteBoardPars {name}): Query<DeleteBoardPars>, State(state): State<Arc<Mutex<AppState>>>) {
    let mut state = state.lock().await;
    state.board_urls.remove(&name);
}


type BoardId = String;
type BoardUrl = String;

struct AppState {
    board_urls: HashMap<BoardId, BoardUrl>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    let state = AppState {
        board_urls: HashMap::new(),
    };
    let app = Router::new()
        .route("/board_url", get(board_url))
        .route("/internal/delete_board", delete(delete_board))
        .with_state(Arc::new(Mutex::new(state)))
        .nest("/board_server", board_server());
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    tracing::info!("starting server");
    axum::serve(listener, app).await.unwrap();
}
