use crate::models;
use crate::AppState;
use axum::{
    extract::{Multipart, State},
    response::{Html, Redirect},
    routing::{get, post},
    Router,
};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use jsonwebtoken::{decode, DecodingKey, Validation};
use memelibre;
use reqwest::Client;
use serde::Serialize;
use std::env;
use std::sync::Arc;
use tera::Context;
use uuid::Uuid;

#[derive(Serialize, sqlx::FromRow)]
struct Meme {
    image_url: String,
    created_at: chrono::DateTime<Utc>,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(html))
        .route("/", post(handler))
}

async fn html(State(state): State<Arc<AppState>>) -> Html<String> {
    let context = Context::new();
    let rendered = state
        .tera
        .render("upload.html", &context)
        .unwrap_or_else(|e| format!("Template error: {}", e));
    Html(rendered)
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    mut multipart: Multipart,
) -> Result<Redirect, Html<String>> {
    let token = match jar.get("token") {
        Some(cookie) => cookie.value().to_string(),
        None => return Ok(Redirect::to("/auth/login")),
    };

    let jwt_secret = env::var("JWT_SECRET").expect("Missing JWT_SECRET env var");

    let token_data = decode::<models::JWTClaims>(
        &token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::default(),
    );

    let claims = match token_data {
        Ok(data) => data.claims,
        Err(_) => return Ok(Redirect::to("/login")),
    };

    let client = Client::new();

    let mut file_data: Option<bytes::Bytes> = None;

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        if field.name() == Some("file") {
            file_data = Some(
                field
                    .bytes()
                    .await
                    .map_err(|_e| Html("Error reading the file".into()))?,
            );
        }
    }

    let file_data = match file_data {
        Some(data) => data,
        None => return Err(Html("Missing file".into())),
    };

    let max_file_size: usize = env::var("MAX_FILE_SIZE")
        .expect("Missing MAX_FILE_SIZE env var")
        .parse()
        .expect("MAX_FILE_SIZE must be a valid number");
    println!(
        "max_file_size: {}, file_data.len(): {}",
        max_file_size,
        file_data.len()
    );

    if file_data.len() > max_file_size {
        println!("in");
        return Err(Html("File is too large (max limit exceeded).".into()));
    }

    let unique_filename = format!("{}", Uuid::new_v4());

    let image_url = format!(
        "https://f{}.backblazeb2.com/file/memelibre/{}",
        env::var("B2_POD").unwrap(),
        unique_filename
    );

    let b2_credentials = memelibre::get_b2_token()
        .await
        .map_err(|e| Html(format!("Failed to get B2 credentials: {}", e)))?;

    let response = client
        .post(b2_credentials.upload_url)
        .header("Authorization", b2_credentials.auth_token)
        .header("X-Bz-File-Name", unique_filename.clone())
        .header("Content-Type", "b2/x-auto")
        .header("Content-Length", file_data.len())
        .header("X-Bz-Content-Sha1", "do_not_verify")
        .body(file_data)
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            sqlx::query("INSERT INTO memes (image_url, created_at) VALUES ($1, $2)")
                .bind(&image_url)
                .bind(Utc::now())
                .execute(&state.pool)
                .await
                .map_err(|e| Html(format!("Database error: {}", e)))?;

            Ok(Redirect::to("/"))
        }
        Ok(resp) => {
            let err_text = resp.text().await.unwrap_or_default();
            Err(Html(format!("Upload failed: {}", err_text)))
        }
        Err(e) => Err(Html(format!("Request error: {}", e))),
    }
}
