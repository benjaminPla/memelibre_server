use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::Serialize;
use std::sync::Arc;

use crate::AppState;

#[derive(Serialize, sqlx::FromRow)]
pub struct Meme {
    id: i32,
    image_url: String,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<Meme>, StatusCode> {
    let meme: Option<Meme> = sqlx::query_as("SELECT id, image_url FROM memes WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| {
            eprintln!("{}:{} - {}", file!(), line!(), e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    meme.map(Json).ok_or(StatusCode::NOT_FOUND)
}
