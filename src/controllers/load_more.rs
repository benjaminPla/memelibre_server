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
    Path(last_id): Path<i32>,
) -> Result<Json<Vec<models::Meme>>, (StatusCode, String)> {
    let memes: Vec<models::Meme> = sqlx::query_as(
        "
        SELECT id, image_url FROM memes
        WHERE id < $1
        ORDER BY id DESC
        LIMIT $2
        ",
    )
    .bind(last_id)
    .bind(&state.config.memes_pull_limit)
    .fetch_all(&state.db)
    .await
    .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?;

    Ok(Json(memes))
}
