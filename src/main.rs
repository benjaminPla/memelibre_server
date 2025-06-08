use axum::{
    response::Redirect,
    routing::{delete,get, post},
    Router,
};
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
    let max_request_size = env::var("MAX_REQUEST_SIZE")
        .map_err(|e| {
            eprintln!("{}:{} - {}", file!(), line!(), e);
            Redirect::to("/error")
        })?
        .parse::<usize>()
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

    let tera = Tera::new("templates/**/*").map_err(|e| {
        eprintln!("{}:{} - {}", file!(), line!(), e);
        Redirect::to("/error")
    })?;

    let app_state = Arc::new(AppState { pool, tera });

    let public_dir = if std::path::Path::new("src/public").exists() {
        "src/public"
    } else {
        "public"
    };
    let app = Router::new()
        .nest_service("/public", ServeDir::new(public_dir))
        .route("/meme/get/all", get(controllers::meme_get_all::handler))
        .route("/meme/get/{id}", get(controllers::meme_get_by_id::handler))
        .route("/meme/post", post(controllers::meme_post::handler))
        .route("/load_more/{id}", get(controllers::load_more::handler))
        .route("/meme/delete/{id}", delete(controllers::delete::handler))
        .with_state(app_state)
        .layer(NormalizePathLayer::trim_trailing_slash())
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(timeout_duration)))
        .layer(RequestBodyLimitLayer::new(max_request_size));

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
