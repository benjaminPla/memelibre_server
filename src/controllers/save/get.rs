use crate::http_error;
use crate::models;
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;

pub async fn handler(
    State(state): State<Arc<models::AppState>>,
    Extension(claims): Extension<models::JWTClaims>,
) -> Result<Json<Vec<models::MemeWithUsername>>, (StatusCode, String)> {
    let saved: Vec<models::MemeWithUsername> = sqlx::query_as(
        "
        SELECT
            memes.created_by,
            memes.id,
            memes.image_url,
            memes.like_count,
            users.username
        FROM saved
        JOIN memes ON saved.meme_id = memes.id
        LEFT JOIN users ON memes.created_by = users.id
        WHERE saved.user_id = $1
        ORDER BY memes.id DESC
        ",
    )
    .bind(claims.sub)
    .fetch_all(&state.db)
    .await
    .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?;

    Ok(Json(saved))
}
