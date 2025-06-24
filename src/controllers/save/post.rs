use crate::http_error;
use crate::models;
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
};
use std::sync::Arc;

pub async fn handler(
    State(state): State<Arc<models::AppState>>,
    Path(meme_id): Path<i32>,
    Extension(claims): Extension<models::JWTClaims>,
) -> Result<StatusCode, (StatusCode, String)> {
    let existing_saved: Option<models::Save> = sqlx::query_as(
        "SELECT meme_id, user_id FROM saved WHERE meme_id = $1 AND user_id = $2",
    )
    .bind(&meme_id)
    .bind(&claims.sub)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?;

    if existing_saved.is_some() {
        sqlx::query("DELETE FROM saved WHERE meme_id = $1 AND user_id = $2")
            .bind(&meme_id)
            .bind(&claims.sub)
            .execute(&state.db)
            .await
            .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?;

        Ok(StatusCode::NO_CONTENT)
    } else {
        sqlx::query("INSERT INTO saved (meme_id, user_id) VALUES ($1, $2)")
            .bind(&meme_id)
            .bind(&claims.sub)
            .execute(&state.db)
            .await
            .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?;

        Ok(StatusCode::CREATED)
    }
}
