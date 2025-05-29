use axum::Router;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::sync::Arc;
use tera::Tera;
use tower_http::services::ServeDir;

mod controllers;

#[derive(Clone)]
pub struct AppState {
    pool: PgPool,
    tera: Tera,
}

#[tokio::main]
async fn main() {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:password@localhost/test")
        .await
        .expect("Error initializing pool");
    let tera = Tera::new("src/html/*").expect("Error initializing Tera");

    let app_state = Arc::new(AppState { pool, tera });

    let app = Router::new()
        .nest_service("/public", ServeDir::new("src/public"))
        .merge(controllers::home::router())
        .with_state(app_state)
        .nest("/health", controllers::health::router());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to address");
    axum::serve(listener, app)
        .await
        .expect("Error initializing server");
}
