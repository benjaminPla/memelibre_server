use axum::Router;
use std::sync::Arc;
use tera::Tera;
use tower_http::services::ServeDir;

mod controllers;

#[tokio::main]
async fn main() {
    let tera = Arc::new(Tera::new("src/html/*").expect("Error initializing Tera"));

    let app = Router::new()
        .nest_service("/styles", ServeDir::new("src/public/styles"))
        .merge(controllers::home::router())
        .with_state(tera)
        .nest("/health", controllers::health::router());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app)
        .await
        .expect("Error initializing server");
}
