use axum::Router;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let app = Router::new().nest_service("/", tower_http::services::ServeDir::new("public"));

    axum::Server::bind(&"127.0.0.1:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
