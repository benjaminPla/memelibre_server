use crate::http_error;
use crate::models;
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct PutUserReq {
    username: String,
}

pub async fn handler(
    State(state): State<Arc<models::AppState>>,
    Extension(claims): Extension<models::JWTClaims>,
    Json(payload): Json<PutUserReq>,
) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query_as::<_, models::User>(
        "
        UPDATE users 
        SET username = $1 
        WHERE id = $2
        RETURNING id, is_admin, username
        ",
    )
    .bind(&payload.username)
    .bind(&claims.sub)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("{:?}", e);
        if e.to_string().contains("duplicate key") {
            http_error!(StatusCode::CONFLICT, "Username already taken")
        } else {
            http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e)
        }
    })?;

    Ok(StatusCode::OK)
}
