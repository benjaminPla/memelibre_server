use axum::Router;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;
use std::sync::Arc;
use tera::Tera;
use tower_http::services::ServeDir;

mod controllers;
mod middlewares;
mod models;

#[derive(Clone)]
pub struct AppState {
    pool: PgPool,
    tera: Tera,
}

#[tokio::main]
async fn main() {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&env::var("DB_CONN").expect("Missing DB_CONN env var"))
        .await
        .expect("Error initializing pool");
    let tera = Tera::new("src/html/*").expect("Error initializing Tera");

    let app_state = Arc::new(AppState { pool, tera });

    let app = Router::new()
        .nest_service("/public", ServeDir::new("src/public"))
        .merge(controllers::home::router())
        .nest("/login", controllers::login::router())
        .nest("/upload", controllers::upload::router())
        .nest("/users", controllers::users::router())
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to address");
    axum::serve(listener, app)
        .await
        .expect("Error initializing server");
}
