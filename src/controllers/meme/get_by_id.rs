use crate::http_error;
use crate::models;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;

pub async fn handler(
    State(state): State<Arc<models::AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<models::Meme>, (StatusCode, String)> {
    let meme: Option<models::Meme> =
        sqlx::query_as("SELECT id, image_url, like_count FROM memes WHERE id = $1")
            .bind(id)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?;

    meme.map(Json).ok_or(http_error!(StatusCode::NOT_FOUND))
}
