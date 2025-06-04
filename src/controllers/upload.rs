use crate::AppState;
use axum::{
    extract::{Multipart, State},
    response::{Html, Redirect},
    routing::{get, post},
    Router,
};
use chrono::Utc;
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

async fn html(State(state): State<Arc<AppState>>) -> Result<Html<String>, Redirect> {
    let context = Context::new();

    let rendered = state
        .tera
        .render("upload.html", &context)
        .unwrap_or_else(|_| "Internal server error".to_string());

    Ok(Html(rendered))
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Redirect, Html<String>> {
    let mut file_data: Option<bytes::Bytes> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| Html("Error processing form".to_string()))?
    {
        if field.name() == Some("file") {
            file_data = Some(
                field
                    .bytes()
                    .await
                    .map_err(|_| Html("Error reading file".to_string()))?,
            );
        }
    }

    let file_data = file_data.ok_or_else(|| Html("Missing file".to_string()))?;

    let max_file_size: usize = env::var("MAX_FILE_SIZE")
        .map_err(|_| Html("Server configuration error".to_string()))?
        .parse()
        .map_err(|_| Html("Server configuration error".to_string()))?;

    if file_data.len() > max_file_size {
        return Err(Html(format!(
            "File is too large (max {} bytes)",
            max_file_size
        )));
    }

    let unique_filename = Uuid::new_v4().to_string();

    let b2_pod = env::var("B2_POD").map_err(|_| Html("Server configuration error".to_string()))?;
    let image_url = format!(
        "https://f{}.backblazeb2.com/file/memelibre/{}",
        b2_pod, unique_filename
    );

    let b2_credentials = match memelibre::get_b2_token().await {
        Ok(creds) => creds,
        Err(e) => {
            return Err(Html(format!("Failed to connect to storage service: {}", e)));
        }
    };

    let client = Client::new();
    let response = client
        .post(&b2_credentials.upload_url)
        .header("Authorization", &b2_credentials.auth_token)
        .header("X-Bz-File-Name", &unique_filename)
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
                .map_err(|e| {
                    eprintln!("{}", e);
                    Html("Failed to save file metadata".to_string())
                })?;

            Ok(Redirect::to("/"))
        }
        Ok(resp) => {
            let err_text = resp
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(Html(format!("Upload failed: {}", err_text)))
        }
        Err(_) => Err(Html("Failed to upload file".to_string())),
    }
}
