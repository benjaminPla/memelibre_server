use crate::models;
use crate::AppState;
use axum::{
    extract::{Form, State},
    http::StatusCode,
    response::{Html, Redirect},
    routing::{get, post},
    Router,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use cookie::time::Duration;
use jsonwebtoken::{encode, EncodingKey, Header};
use memelibre;
use serde::Deserialize;
use std::env;
use std::sync::Arc;
use tera::Context;
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
    Router::new().route("/", get(html)).route("/", post(login))
}

async fn html(State(state): State<Arc<AppState>>) -> Html<String> {
    let context = Context::new();
    let rendered = state
        .tera
        .render("login.html", &context)
        .unwrap_or_else(|e| format!("Template error: {}", e));
    Html(rendered)
}

async fn login(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Form(body): Form<LoginRequest>,
) -> Result<(CookieJar, Redirect), StatusCode> {
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
        None => return Err(StatusCode::UNAUTHORIZED),
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

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    ) {
        Ok(token) => token,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let cookie = Cookie::build(("token", token))
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .max_age(Duration::seconds(3600))
        .build();

    Ok((jar.add(cookie), Redirect::to("/")))
}
