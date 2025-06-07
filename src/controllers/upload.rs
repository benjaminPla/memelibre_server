use crate::AppState;
use axum::{
    extract::{Multipart, State},
    response::{Html, Redirect},
    routing::{get, post},
    Router,
};
use chrono::Utc;
use image::{ImageFormat, ImageReader};
use memelibre;
use reqwest::Client;
use serde::Serialize;
use std::env;
use std::io::Cursor;
use std::sync::Arc;
use tera::Context;
use webp::Encoder;

#[derive(Serialize, sqlx::FromRow)]
struct Meme {
    image_url: String,
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
    let b2_pod = env::var("B2_POD").map_err(|_| Html("Server configuration error".to_string()))?;
    let compression_quality: f32 = env::var("COMPRESSION_QUALITY")
        .map_err(|_| Html("Server configuration error".to_string()))?
        .parse::<f32>()
        .map_err(|_| Html("Server configuration error".to_string()))?
        .clamp(0.0, 100.0);

    let mut file_data: Option<bytes::Bytes> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        eprintln!("{}", e);
        Html("Error reading file".to_string())
    })? {
        if field.name() == Some("file") {
            file_data = Some(field.bytes().await.map_err(|e| {
                eprintln!("{}", e);
                Html("Error reading file".to_string())
            })?);
        }
    }

    let file_data = file_data.ok_or_else(|| Html("Missing file".to_string()))?;

    let guessed_format = ImageReader::new(Cursor::new(&file_data))
        .with_guessed_format()
        .map_err(|_| Html("Unsupported image format".to_string()))?
        .format();

    let (data, extension) = if guessed_format == Some(ImageFormat::Gif) {
        (file_data.to_vec(), "gif")
    } else {
        let img = ImageReader::new(Cursor::new(&file_data))
            .with_guessed_format()
            .map_err(|_| Html("Unsupported image format".to_string()))?
            .decode()
            .map_err(|_| Html("Failed to decode image".to_string()))?;

        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();

        let webp_data = Encoder::from_rgba(&rgba, width, height)
            .encode(compression_quality)
            .to_vec();
        (webp_data, "webp")
    };

    let timestamp = Utc::now().format("%Y-%m-%d-%H:%M:%S%.3f").to_string();
    let unique_filename = format!("{}.{}", timestamp, extension);

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
        .header("Content-Length", data.len())
        .header("Content-Type", "b2/x-auto")
        .header("X-Bz-Content-Sha1", "do_not_verify")
        .header("X-Bz-File-Name", &unique_filename)
        .body(data.clone())
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            sqlx::query("INSERT INTO memes (image_url) VALUES ($1)")
                .bind(&image_url)
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
