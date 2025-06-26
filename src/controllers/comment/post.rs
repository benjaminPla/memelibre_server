use crate::http_error;
use crate::models;
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct PostCommentReq {
    content: String,
}

pub async fn handler(
    State(state): State<Arc<models::AppState>>,
    Extension(claims): Extension<models::JWTClaims>,
    Path(meme_id): Path<i32>,
    Json(payload): Json<PostCommentReq>,
) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query("INSERT INTO comments (meme_id, user_id, content) VALUES ($1, $2, $3)")
        .bind(&meme_id)
        .bind(&claims.sub)
        .bind(&payload.content)
        .execute(&state.db)
        .await
        .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?;

    Ok(StatusCode::CREATED)
}
