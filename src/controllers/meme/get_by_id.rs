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
) -> Result<Json<models::MemeWithUsername>, (StatusCode, String)> {
    let meme: Option<models::MemeWithUsername> =
        sqlx::query_as(
            "
            SELECT
                memes.created_by,
                memes.id,
                memes.image_url,
                memes.like_count,
                users.username
            FROM memes
            LEFT JOIN users ON memes.created_by = users.id
            WHERE memes.id = $1
            "
            )
            .bind(id)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?;

    meme.map(Json).ok_or(http_error!(StatusCode::NOT_FOUND))
}
