use crate::AppState;
use axum::{extract::State, http::StatusCode, response::Json};
use serde::Serialize;
use std::{env, sync::Arc};

#[derive(Serialize, sqlx::FromRow)]
pub struct Meme {
    id: i32,
    image_url: String,
}

pub async fn handler(State(state): State<Arc<AppState>>) -> Result<Json<Vec<Meme>>, StatusCode> {
    let limit = env::var("MEMES_PULL_LIMIT")
        .map_err(|e| {
            eprintln!("{}:{} - {}", file!(), line!(), e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .parse::<i64>()
        .map_err(|e| {
            eprintln!("{}:{} - {}", file!(), line!(), e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let memes = sqlx::query_as::<_, Meme>(
        "
        SELECT id, image_url
        FROM memes
        ORDER BY id DESC
        LIMIT $1
        ",
    )
    .bind(limit)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        eprintln!("{}:{} - {}", file!(), line!(), e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(memes))
}
