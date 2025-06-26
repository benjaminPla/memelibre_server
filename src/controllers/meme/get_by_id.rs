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
) -> Result<Json<models::MemeWithUsernameAndComments>, (StatusCode, String)> {
    let meme: Option<models::MemeWithUsername> = sqlx::query_as(
        "
            SELECT
                memes.id,
                memes.image_url,
                memes.like_count,
                users.username
            FROM memes
            LEFT JOIN users ON memes.created_by = users.id
            WHERE memes.id = $1
            ",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?;

    let meme = meme.ok_or(http_error!(StatusCode::NOT_FOUND))?;

    let comments: Vec<models::CommentWithUsername> = sqlx::query_as(
        "
        SELECT 
            comments.content,
            comments.id,
            comments.meme_id,
            users.username
        FROM comments
        LEFT JOIN users ON comments.user_id = users.id
        WHERE comments.meme_id = $1
        ORDER BY comments.id ASC
        ",
    )
    .bind(id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?;

    let result = models::MemeWithUsernameAndComments {
        id: meme.id,
        image_url: meme.image_url,
        like_count: meme.like_count,
        username: meme.username,
        comments,
    };

    Ok(Json(result))
}
