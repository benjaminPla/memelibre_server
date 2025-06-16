use crate::http_error;
use crate::models::Meme;
use crate::AppState;
use axum::{extract::State, http::StatusCode, response::Json};
use std::sync::Arc;

pub async fn handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Meme>>, (StatusCode, String)> {
    let memes = sqlx::query_as::<_, Meme>(
        "
        SELECT id, image_url
        FROM memes
        ORDER BY id DESC
        LIMIT $1
        ",
    )
    .bind(&state.config.memes_pull_limit)
    .fetch_all(&state.db)
    .await
    .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?;

    Ok(Json(memes))
}
