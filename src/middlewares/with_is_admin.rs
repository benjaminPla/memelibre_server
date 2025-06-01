use axum::{
    body::Body,
    extract::Extension,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::models;

pub async fn handler(
    Extension(claims): Extension<models::JWTClaims>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    if !claims.is_admin {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(req).await)
}
