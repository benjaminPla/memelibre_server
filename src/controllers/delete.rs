use crate::http_error;
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use memelibre_server::create_bucket_client;
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize, sqlx::FromRow)]
struct Meme {
    id: i32,
    image_url: String,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<StatusCode, (StatusCode, String)> {
    let meme = sqlx::query_as::<_, Meme>("SELECT id, image_url FROM memes WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?
        .ok_or(http_error!(StatusCode::NOT_FOUND))?;

    let object_key = meme
        .image_url
        .rsplit('/')
        .next()
        .ok_or(http_error!(StatusCode::NOT_FOUND))?;

    let bucket_client = create_bucket_client()
        .await
        .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?;

    bucket_client
        .delete_object()
        .bucket(&state.config.bucket_name)
        .key(object_key)
        .send()
        .await
        .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?;

    let meme_deleted = sqlx::query("DELETE FROM memes WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?;

    if meme_deleted.rows_affected() == 0 {
        return Err(http_error!(StatusCode::NOT_FOUND));
    }

    Ok(StatusCode::NO_CONTENT)
}
