use crate::AppState;
use axum::{
    extract::{Json, State},
    response::Redirect,
};
use memelibre;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct Request {
    email: String,
    is_admin: bool,
    password: String,
    username: String,
}

pub async fn handler(State(state): State<Arc<AppState>>, Json(payload): Json<Request>) -> Redirect {
    let hashed_password = memelibre::hash_password(&payload.password);

    let result = sqlx::query(
        "INSERT INTO users (email, hashed_password, is_admin, username)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(&payload.email)
    .bind(&hashed_password)
    .bind(&payload.is_admin)
    .bind(&payload.username)
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => Redirect::to("/"),
        Err(e) => {
            eprintln!("Failed to insert user: {e}");
            Redirect::to("/error")
        }
    }
}
