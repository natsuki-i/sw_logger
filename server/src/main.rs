use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{prelude::*, SinkExt};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let state = Arc::new(AppState::new());
    let app = Router::new()
        .route("/socket", get(websocket_handler))
        .route("/push", get(push_handler))
        .with_state(state)
        .nest_service("/", tower_http::services::ServeDir::new("public"));

    axum::Server::bind(&"127.0.0.1:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

struct AppState {
    tx: broadcast::Sender<Message>,
}

impl AppState {
    pub fn new() -> AppState {
        let (tx, _) = broadcast::channel(100);
        AppState { tx }
    }
}

async fn push_handler(
    Query(query): Query<HashMap<String, f64>>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match serde_json::to_string(&query) {
        Ok(s) => {
            state.tx.send(Message::Text(s)).ok();
            "OK".into()
        }
        Err(e) => format!("failed to encode json: {}", e),
    }
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket_worker(socket, state))
}

async fn websocket_worker(stream: WebSocket, state: Arc<AppState>) {
    let rx = BroadcastStream::new(state.tx.subscribe());
    rx.map_err(|e| log::info!("{}", e))
        .forward(stream.sink_map_err(|e| log::info!("{}", e)))
        .await
        .ok();
}
