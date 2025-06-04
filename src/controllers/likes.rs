use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    routing::post,
    Router,
};
use std::sync::Arc;

use crate::middlewares;
use crate::models::JWTClaims;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/{meme_id}", post(handler))
        .layer(axum::middleware::from_fn(middlewares::with_auth::handler))
}

async fn handler(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<JWTClaims>,
    Path(meme_id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("INSERT INTO likes (user_id, meme_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
        .bind(claims.sub)
        .bind(meme_id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}
