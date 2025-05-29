use axum::http::StatusCode;
use axum::{routing::get, Router};

pub fn router() -> Router {
    Router::new().route("/", get(health))
}

async fn health() -> StatusCode {
    StatusCode::OK
}
