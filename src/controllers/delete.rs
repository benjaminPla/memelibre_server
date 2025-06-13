use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use memelibre_server::create_bucket_client;
use std::sync::Arc;

use serde::Serialize;

#[derive(Serialize, sqlx::FromRow)]
struct Meme {
    id: i32,
    image_url: String,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    let meme = sqlx::query_as::<_, Meme>("SELECT id, image_url FROM memes WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| {
            eprintln!("{}:{} - {}", file!(), line!(), e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let object_key = meme.image_url.rsplit('/').next().unwrap_or("");

    let bucket_client = create_bucket_client().await.map_err(|e| {
        eprintln!("{}:{} - {}", file!(), line!(), e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    bucket_client
        .delete_object()
        .bucket(&state.config.bucket_name)
        .key(object_key)
        .send()
        .await
        .map_err(|e| {
            eprintln!("{}:{} - {}", file!(), line!(), e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let meme_deleted = sqlx::query("DELETE FROM memes WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| {
            eprintln!("{}:{} - {}", file!(), line!(), e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if meme_deleted.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}
