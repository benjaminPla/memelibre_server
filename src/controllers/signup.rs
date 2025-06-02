use crate::AppState;
use axum::{
    extract::{Form, State},
    http::StatusCode,
    response::{Html, Redirect},
    routing::{get, post},
    Router,
};
use memelibre;
use serde::Deserialize;
use std::sync::Arc;
use tera::Context;

#[derive(Deserialize)]
struct LoginRequest {
    password: String,
    username: String,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", get(html)).route("/", post(signup))
}

async fn html(State(state): State<Arc<AppState>>) -> Html<String> {
    let context = Context::new();
    let rendered = state
        .tera
        .render("signup.html", &context)
        .unwrap_or_else(|e| format!("Template error: {}", e));
    Html(rendered)
}

async fn signup(
    State(state): State<Arc<AppState>>,
    Form(form): Form<LoginRequest>,
) -> Result<Redirect, StatusCode> {
    let hashed_password = memelibre::hash_password(&form.password);

    let result = sqlx::query(
        "INSERT INTO users (hashed_password,username)
         VALUES ($1, $2)",
    )
    .bind(&hashed_password)
    .bind(&form.username)
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => Ok(Redirect::to("/login")),
        Err(e) => {
            eprintln!("Failed to insert user: {e}");
            Ok(Redirect::to("/error"))
        }
    }
}
