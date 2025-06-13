use crate::AppState;
use axum::{extract::State, http::StatusCode, response::Json};
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize, sqlx::FromRow)]
pub struct Meme {
    id: i32,
    image_url: String,
}

pub async fn handler(State(state): State<Arc<AppState>>) -> Result<Json<Vec<Meme>>, StatusCode> {
    let memes = sqlx::query_as::<_, Meme>(
        "
        SELECT id, image_url
        FROM memes
        ORDER BY id DESC
        LIMIT $1
        ",
    )
    .bind(&state.config.memes_pull_limit)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        eprintln!("{}:{} - {}", file!(), line!(), e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(memes))
}
