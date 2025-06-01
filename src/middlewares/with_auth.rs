use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use std::env;

use crate::models;

pub async fn handler(mut req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    let token = match req.headers().get("Authorization") {
        Some(token) => token,
        None => return Err(StatusCode::UNAUTHORIZED),
    };
    println!("0. {:?}", token);
    let token = match token.to_str() {
        Ok(token) => token,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    println!("1. {}", token);

    let jwt_secret = env::var("JWT_SECRET").expect("Missing JWT_SECRET env var");

    let token_data = match decode::<models::JWTClaims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("2. {}", e);
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    req.extensions_mut().insert(token_data.claims);

    Ok(next.run(req).await)
}
