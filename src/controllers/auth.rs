use crate::AppState;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use memelibre;
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub is_admin: bool,
    pub exp: usize,
}

#[derive(sqlx::FromRow, Debug)]
struct User {
    id: Uuid,
    email: String,
    hashed_password: String,
    is_admin: bool,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", post(handler))
}

async fn handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LoginRequest>,
) -> Result<Response, StatusCode> {
    let user: Result<Option<User>, _> =
        sqlx::query_as("SELECT id, email, hashed_password, is_admin FROM users WHERE email = $1")
            .bind(&body.email)
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

    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET not set");
    let claims = Claims {
        sub: user.id.to_string(),
        is_admin: user.is_admin,
        exp: (chrono::Utc::now().timestamp() + 3600) as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::OK, Json(token)).into_response())
}
