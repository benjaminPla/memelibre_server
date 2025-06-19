use crate::http_error;
use crate::models;
use axum::{
    body::Body,
    extract::{Extension, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

pub async fn handler(
    State(_state): State<Arc<models::AppState>>,
    Extension(claims): Extension<models::JWTClaims>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    if !claims.is_admin {
        return Err(http_error!(StatusCode::UNAUTHORIZED));
    }

    Ok(next.run(req).await)
}
