use axum::{
    body::Body,
    http::Request,
    middleware::Next,
    response::{Redirect, Response},
};
use axum_extra::extract::cookie::CookieJar;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use std::env;

use crate::models::JWTClaims;

pub async fn handler(
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, Redirect> {
    let jar = CookieJar::from_headers(req.headers());

    let token = jar
        .get("token")
        .ok_or_else(|| Redirect::to("/login"))?
        .value();

    let jwt_secret = env::var("JWT_SECRET").map_err(|_| Redirect::to("/error"))?;
    let claims = decode::<JWTClaims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|_| Redirect::to("/login"))?
    .claims;

    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}

