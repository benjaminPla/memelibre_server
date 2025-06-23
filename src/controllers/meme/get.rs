use crate::http_error;
use crate::models;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;

pub async fn handler(
    State(state): State<Arc<models::AppState>>,
    Query(params): Query<models::Pagination>,
) -> Result<Json<Vec<models::MemeWithUsername>>, (StatusCode, String)> {
    let memes: Vec<models::MemeWithUsername> = sqlx::query_as(
        "
        SELECT
            memes.created_by,
            memes.id,
            memes.image_url,
            memes.like_count,
            users.username
        FROM memes
        LEFT JOIN users ON memes.created_by = users.id
        WHERE memes.id < COALESCE($1, 2147483647)
        ORDER BY memes.id DESC
        LIMIT $2;
        ",
    )
    .bind(params.offset)
    .bind(&state.config.memes_pull_limit)
    .fetch_all(&state.db)
    .await
    .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?;

    Ok(Json(memes))
}
