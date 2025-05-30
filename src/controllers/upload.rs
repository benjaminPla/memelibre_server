use axum::{
    extract::{Multipart, State},
    response::{Html, Redirect},
    routing::{get, post},
    Router,
};
use chrono::Utc;
use reqwest::Client;
use serde::Serialize;
use std::env;
use std::sync::Arc;
use tera::Context;
use uuid::Uuid;

use crate::AppState;

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
    mut multipart: Multipart,
) -> Result<Redirect, Html<String>> {
    let client = Client::new();

    let mut file_data: Option<bytes::Bytes> = None;

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        if field.name() == Some("file") {
            file_data = Some(field.bytes().await.unwrap_or_default());
        }
    }

    let file_data = match file_data {
        Some(data) => data,
        None => return Err(Html("Missing file".into())),
    };

    let unique_filename = format!("{}", Uuid::new_v4());

    let image_url = format!(
        "https://f{}.backblazeb2.com/file/memelibre/{}",
        env::var("B2_POD").unwrap(),
        unique_filename
    );

    let upload_url = dbg!(env::var("B2_UPLOAD_URL").unwrap());
    let upload_auth_token = dbg!(env::var("B2_TOKEN").unwrap());

    let response = client
        .post(upload_url)
        .header("Authorization", upload_auth_token)
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
