use crate::models;
use crate::AppState;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use memelibre;
use serde::Deserialize;
use std::env;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
struct LoginRequest {
    password: String,
    username: String,
}

#[derive(sqlx::FromRow)]
struct User {
    hashed_password: String,
    id: Uuid,
    is_admin: bool,
    username: String,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", post(handler))
}

async fn handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LoginRequest>,
) -> Result<Response, StatusCode> {
    let user: Result<Option<User>, _> = sqlx::query_as(
        "SELECT hashed_password, id, is_admin, username FROM users WHERE username = $1",
    )
    .bind(&body.username)
    .fetch_optional(&state.pool)
    .await;

    let user = match user {
        Ok(user) => user,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let user = match user {
        Some(user) => user,
        None => return Err(StatusCode::NOT_FOUND),
    };

    if !memelibre::verify_password(&user.hashed_password, &body.password) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let jwt_secret = env::var("JWT_SECRET").expect("Missing JWT_SECRET env var");

    let claims = models::JWTClaims {
        exp: (chrono::Utc::now().timestamp() + 3600) as usize,
        is_admin: user.is_admin,
        sub: user.id,
        username: user.username,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::OK, Json(token)).into_response())
}
