use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::{Json, State},
    response::Redirect,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::AppState;

#[derive(Deserialize)]
pub struct Request {
    email: String,
    password: String,
    username: String,
}

pub async fn handler(State(state): State<Arc<AppState>>, Json(payload): Json<Request>) -> Redirect {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let hashed_password = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .expect("Error on `hash_password`")
        .to_string();

    let result = sqlx::query(
        "INSERT INTO users (email, username, hashed_password)
         VALUES ($1, $2, $3)",
    )
    .bind(&payload.email)
    .bind(&payload.username)
    .bind(&hashed_password)
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
