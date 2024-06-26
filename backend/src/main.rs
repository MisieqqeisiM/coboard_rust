mod socket_endpoint;
mod board_server;
mod board;

use std::{collections::HashMap, sync::Arc};

use axum::{extract::{Query, State}, response::{IntoResponse, Response}, routing::get, Router};
use axum_macros::debug_handler;
use board_server::board_server;
use serde::Deserialize;
use tokio::{net::TcpListener, sync::Mutex};

const INNER_BOARD_SERVER_URL: &'static str = "http://localhost:8080/board_server";
const OUTER_BOARD_SERVER_URL: &'static str = "/api/board_server";

#[derive(Deserialize)]
struct BoardUrlPars {
    name: String,
}

#[debug_handler]
async fn board_url(Query(BoardUrlPars {name}): Query<BoardUrlPars>, State(state): State<Arc<Mutex<AppState>>>) -> Response {
    let mut state = state.lock().await;
    match state.board_urls.get(&name) {
        Some(url) => url.to_owned().into_response(),
        None => {
            let path = reqwest::get(format!("{}/create_board", INNER_BOARD_SERVER_URL)).await.unwrap().text().await.unwrap();
            let path = format!("{}{}", OUTER_BOARD_SERVER_URL, path);
            state.board_urls.insert(name, path.clone());
            path.into_response()
        }
    }
}

struct AppState {
    board_urls: HashMap<String, String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    let state = AppState {
        board_urls: HashMap::new(),
    };
    let app = Router::new()
        .route("/board_url", get(board_url))
        .with_state(Arc::new(Mutex::new(state)))
        .nest("/board_server", board_server());
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    tracing::info!("starting server");
    axum::serve(listener, app).await.unwrap();
}
