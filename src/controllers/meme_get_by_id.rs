use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use memelibre_server::internal_error;
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
) -> Result<Json<Meme>, (StatusCode, String)> {
    let meme: Option<Meme> = sqlx::query_as("SELECT id, image_url FROM memes WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?;

    meme.map(Json)
        .ok_or((StatusCode::NOT_FOUND, "Not found".to_string()))
}
