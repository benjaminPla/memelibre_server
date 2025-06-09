use axum::{
    response::Redirect,
    routing::{delete, get, post},
    Router,
};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, timeout::TimeoutLayer};

mod controllers;

#[derive(Clone)]
pub struct AppState {
    pool: PgPool,
}

#[tokio::main]
async fn main() -> Result<(), Redirect> {
    let db_conn_string = env::var("DB_CONN_STRING").map_err(|e| {
        eprintln!("{}:{} - {}", file!(), line!(), e);
        Redirect::to("/error")
    })?;
    let db_max_conn = env::var("DB_MAX_CONN")
        .map_err(|e| {
            eprintln!("{}:{} - {}", file!(), line!(), e);
            Redirect::to("/error")
        })?
        .parse::<u32>()
        .map_err(|e| {
            eprintln!("{}:{} - {}", file!(), line!(), e);
            Redirect::to("/error")
        })?;
    let timeout_duration = env::var("TIMEOUT_DURATION")
        .map_err(|e| {
            eprintln!("{}:{} - {}", file!(), line!(), e);
            Redirect::to("/error")
        })?
        .parse::<u64>()
        .map_err(|e| {
            eprintln!("{}:{} - {}", file!(), line!(), e);
            Redirect::to("/error")
        })?;

    let pool = PgPoolOptions::new()
        .max_connections(db_max_conn)
        .connect(&db_conn_string)
        .await
        .map_err(|e| {
            eprintln!("{}:{} - {}", file!(), line!(), e);
            Redirect::to("/error")
        })?;

    let app_state = Arc::new(AppState { pool });

    let app = Router::new()
        .nest(
            "/api",
            Router::new()
                .route("/meme/get", get(controllers::meme_get_all::handler))
                .route("/meme/get/:id", get(controllers::meme_get_by_id::handler))
                .route("/meme/post", post(controllers::meme_post::handler))
                .route("/meme/delete/:id", delete(controllers::delete::handler))
                .route("/load_more/:id", get(controllers::load_more::handler))
                .with_state(app_state),
        )
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(timeout_duration)));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .map_err(|e| {
            eprintln!("{}:{} - {}", file!(), line!(), e);
            Redirect::to("/error")
        })?;
    axum::serve(listener, app).await.map_err(|e| {
        eprintln!("{}:{} - {}", file!(), line!(), e);
        Redirect::to("/error")
    })?;

    Ok(())
}
