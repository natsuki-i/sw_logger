mod args;
mod values;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, RawQuery, State,
    },
    http::{Request, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use clap::Parser;
use futures::{prelude::*, SinkExt};
use std::{collections::HashMap, net::SocketAddrV4, ops::Deref, sync::Arc};
use tokio::sync::{broadcast, Mutex};
use tokio_stream::wrappers::BroadcastStream;

#[tokio::main]
async fn main() {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let args = args::Args::parse();

    let state = Arc::new(AppState::new());
    let app = Router::new()
        .route("/socket", get(websocket_handler))
        .route("/push", get(push_handler))
        .route("/p", get(push_handler2))
        .route("/download.json", get(download_json))
        .with_state(state)
        .nest_service("/", tower_http::services::ServeDir::new("public"))
        .layer(axum::middleware::from_fn(access_log));

    axum::Server::bind(&SocketAddrV4::new(args.ip, args.port).into())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn access_log<B>(
    req: Request<B>,
    next: axum::middleware::Next<B>,
) -> Result<axum::response::Response, StatusCode> {
    log::info!("{} {}", req.method(), req.uri());
    Ok(next.run(req).await)
}

struct AppState {
    tx: broadcast::Sender<Message>,
    values: Mutex<values::Values>,
}

impl AppState {
    pub fn new() -> AppState {
        let (tx, _) = broadcast::channel(100);
        AppState {
            tx,
            values: Default::default(),
        }
    }
}

async fn push_handler(
    Query(query): Query<Vec<(String, f32)>>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let mut map = HashMap::<String, Vec<f32>>::new();
    let mut values = state.values.lock().await;
    for (k, v) in query {
        values.push(k.clone(), &[v]);
        map.entry(k).or_default().push(v);
    }
    match serde_json::to_string(&map) {
        Ok(s) => {
            state.tx.send(Message::Text(s)).ok();
            "OK".into()
        }
        Err(e) => format!("failed to encode json: {}", e),
    }
}

async fn push_handler2(
    RawQuery(query): RawQuery,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    use base64::prelude::*;
    let v = match BASE64_URL_SAFE_NO_PAD.decode(query.unwrap_or_default()) {
        Ok(v) => v,
        Err(e) => return format!("failed to decode base64: {}", e),
    };
    let v = match rmp_serde::from_slice::<HashMap<String, Vec<f32>>>(&v) {
        Ok(v) => v,
        Err(e) => return format!("failed to decode message pack: {}", e),
    };
    let mut values = state.values.lock().await;
    for (k, v) in &v {
        values.push(k.clone(), v);
    }
    match serde_json::to_string(&v) {
        Ok(s) => {
            state.tx.send(Message::Text(s)).ok();
            "OK".into()
        }
        Err(e) => format!("failed to encode json: {}", e),
    }
}

async fn download_json(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let values = state.values.lock().await;
    let values = serde_json::to_value(values.deref()).unwrap();
    Json(values)
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
