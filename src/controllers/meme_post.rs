use crate::AppState;
use axum::{
    extract::{Multipart, State},
    http::status::StatusCode,
};
use chrono::Utc;
use image::{ImageFormat, ImageReader};
use memelibre;
use reqwest::Client;
use serde::Serialize;
use std::env;
use std::io::Cursor;
use std::sync::Arc;
use webp::Encoder;

#[derive(Serialize, sqlx::FromRow)]
struct Meme {
    image_url: String,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let b2_pod = env::var("B2_POD").map_err(|e| {
        eprintln!("{}:{} - {}", file!(), line!(), e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error".to_string(),
        )
    })?;
    let compression_quality: f32 = env::var("COMPRESSION_QUALITY")
        .map_err(|e| {
            eprintln!("{}:{} - {}", file!(), line!(), e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            )
        })?
        .parse::<f32>()
        .map_err(|e| {
            eprintln!("{}:{} - {}", file!(), line!(), e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            )
        })?
        .clamp(0.0, 100.0);

    let mut file_data: Option<bytes::Bytes> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        eprintln!("{}:{} - {}", file!(), line!(), e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error".to_string(),
        )
    })? {
        if field.name() == Some("file") {
            file_data = Some(field.bytes().await.map_err(|e| {
                eprintln!("{}:{} - {}", file!(), line!(), e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            })?);
        }
    }

    let file_data = file_data.ok_or((StatusCode::BAD_REQUEST, "File is empty".to_string()))?;

    let guessed_format = ImageReader::new(Cursor::new(&file_data))
        .with_guessed_format()
        .map_err(|e| {
            eprintln!("{}:{} - {}", file!(), line!(), e);
            (StatusCode::BAD_REQUEST, "Invalid format".to_string())
        })?
        .format();

    let (data, extension) = if guessed_format == Some(ImageFormat::Gif) {
        (file_data.to_vec(), "gif")
    } else {
        let img = ImageReader::new(Cursor::new(&file_data))
            .with_guessed_format()
            .map_err(|e| {
                eprintln!("{}:{} - {}", file!(), line!(), e);
                (StatusCode::BAD_REQUEST, "Invalid format".to_string())
            })?
            .decode()
            .map_err(|e| {
                eprintln!("{}:{} - {}", file!(), line!(), e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            })?;

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

    let b2_credentials = memelibre::get_b2_token().await.map_err(|e| {
        eprintln!("{}:{} - {}", file!(), line!(), e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error".to_string(),
        )
    })?;

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
                    eprintln!("{}:{} - {}", file!(), line!(), e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Internal server error".to_string(),
                    )
                })?;

            Ok((StatusCode::CREATED, "Upload successful".to_string()))
        }

        _ => {
            eprintln!("{}:{} - upload failed", file!(), line!());
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ))
        }
    }
}
