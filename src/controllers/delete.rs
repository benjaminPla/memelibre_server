use axum::{
    extract::{Path, State},
    http::status::StatusCode,
    routing::delete,
    Router,
};
use serde::Serialize;
use std::sync::Arc;

use crate::AppState;

#[derive(Serialize, sqlx::FromRow)]
struct Meme {
    id: i32,
    image_url: String,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/{id}", delete(handler))
}

async fn handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    let meme_deleted = sqlx::query("DELETE FROM memes WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|e| {
            eprintln!("{}:{} - {}", file!(), line!(), e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if meme_deleted.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    // delete from bucket too
    Ok(StatusCode::NO_CONTENT)
}
