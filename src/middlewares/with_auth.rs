use crate::http_error;
use crate::models;
use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::cookie::CookieJar;
use jsonwebtoken::{decode, errors::ErrorKind, Algorithm, DecodingKey, Validation};
use std::sync::Arc;

pub async fn handler(
    State(state): State<Arc<models::AppState>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let jar = CookieJar::from_headers(req.headers());
    let session_token = jar
        .get("session_token")
        .ok_or_else(|| http_error!(StatusCode::UNAUTHORIZED))?
        .value();

    let claims = decode::<models::JWTClaims>(
        session_token,
        &DecodingKey::from_secret(&state.config.jwt_secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|e| {
        let message = match e.kind() {
            ErrorKind::InvalidToken => "Invalid token".to_string(),
            ErrorKind::InvalidSignature => "Invalid signature".to_string(),
            ErrorKind::ExpiredSignature => "Token has expired".to_string(),
            ErrorKind::InvalidAlgorithm => "Invalid algorithm".to_string(),
            ErrorKind::MissingRequiredClaim(claim) => format!("Missing required claim: {}", claim),
            _ => "JWT Error".to_string(),
        };
        http_error!(StatusCode::UNAUTHORIZED, message)
    })?
    .claims;

    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}
