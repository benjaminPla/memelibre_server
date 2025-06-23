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
) -> Result<Json<models::User>, (StatusCode, String)> {
    let user: models::User = sqlx::query_as(
        "
        SELECT id, is_admin, username
        FROM users
        WHERE id = $1
        ",
    )
    .bind(&claims.sub)
    .fetch_one(&state.db)
    .await
    .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?;

    Ok(Json(user))
}
