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
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?;

    let existing_like: Option<models::Like> =
        sqlx::query_as("SELECT meme_id, user_id FROM likes WHERE meme_id = $1 AND user_id = $2")
            .bind(&meme_id)
            .bind(&claims.sub)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?;

    if existing_like.is_some() {
        sqlx::query("DELETE FROM likes WHERE meme_id = $1 AND user_id = $2")
            .bind(&meme_id)
            .bind(&claims.sub)
            .execute(&mut *tx)
            .await
            .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?;

        sqlx::query("UPDATE memes SET like_count = like_count - 1 WHERE id = $1")
            .bind(&meme_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?;

        tx.commit()
            .await
            .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?;

        Ok(StatusCode::NO_CONTENT)
    } else {
        sqlx::query("INSERT INTO likes (meme_id, user_id) VALUES ($1, $2)")
            .bind(&meme_id)
            .bind(&claims.sub)
            .execute(&mut *tx)
            .await
            .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?;

        sqlx::query("UPDATE memes SET like_count = like_count + 1 WHERE id = $1")
            .bind(&meme_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?;

        tx.commit()
            .await
            .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?;

        Ok(StatusCode::CREATED)
    }
}
