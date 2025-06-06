use axum::Router;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tera::Tera;
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer, limit::RequestBodyLimitLayer,
    normalize_path::NormalizePathLayer, services::ServeDir, timeout::TimeoutLayer,
};

mod controllers;

#[derive(Clone)]
pub struct AppState {
    pool: PgPool,
    tera: Tera,
}

#[tokio::main]
async fn main() {
    let db_conn_string = env::var("DB_CONN_STRING").expect("Missing DB_CONN_STRING env var");
    let db_max_conn = env::var("DB_MAX_CONN")
        .expect("Missing DB_MAX_CONN env var")
        .parse::<u32>()
        .expect("Error parsing DB_MAX_CONN env var");
    let max_request_size = env::var("MAX_REQUEST_SIZE")
        .expect("Missing MAX_REQUEST_SIZE env var")
        .parse::<usize>()
        .expect("Error parsing MAX_REQUEST_SIZE env var");
    let timeout_duration = env::var("TIMEOUT_DURATION")
        .expect("Missing TIMEOUT_DURATION env var")
        .parse::<u64>()
        .expect("Error parsing TIMEOUT_DURATION env var");

    let pool = PgPoolOptions::new()
        .max_connections(db_max_conn)
        .connect(&db_conn_string)
        .await
        .expect("Error initializing pool");

    let tera = Tera::new("templates/**/*").expect("Error initializing Tera");

    let app_state = Arc::new(AppState { pool, tera });

    let public_dir = if std::path::Path::new("src/public").exists() {
        "src/public"
    } else {
        "public"
    };
    let app = Router::new()
        .nest_service("/public", ServeDir::new(public_dir))
        .merge(controllers::home::router())
        .nest("/meme", controllers::meme::router())
        .nest("/load_more", controllers::load_more::router())
        .nest("/upload", controllers::upload::router())
        .with_state(app_state)
        .layer(NormalizePathLayer::trim_trailing_slash())
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(timeout_duration)))
        .layer(RequestBodyLimitLayer::new(max_request_size));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to address");
    axum::serve(listener, app)
        .await
        .expect("Error initializing server");
}
