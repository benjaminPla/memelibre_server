use crate::AppState;
use aws_sdk_s3::primitives::ByteStream;
use axum::{
    extract::{Multipart, State},
    http::status::StatusCode,
};
use chrono::Utc;
use image::{ImageFormat, ImageReader};
use memelibre_server::{create_bucket_client, internal_error};
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
    let bucket_endpoint = env::var("BUCKET_ENDPOINT").map_err(internal_error)?;
    let bucket_name = env::var("BUCKET_NAME").map_err(internal_error)?;
    let bucket_object_max_size = env::var("BUCKET_OBJECT_MAX_SIZE")
        .map_err(internal_error)?
        .parse::<usize>()
        .map_err(internal_error)?;

    let compression_quality: f32 = env::var("COMPRESSION_QUALITY")
        .map_err(internal_error)?
        .parse::<f32>()
        .map_err(internal_error)?
        .clamp(0.0, 100.0);

    let mut file_data: Option<bytes::Bytes> = None;

    while let Some(field) = multipart.next_field().await.map_err(internal_error)? {
        if field.name() == Some("file") {
            let data = field.bytes().await.map_err(internal_error)?;
            if data.len() > bucket_object_max_size {
                return Err((
                    StatusCode::BAD_REQUEST,
                    format!("File size exceeds maximum limit"),
                ));
            }
            file_data = Some(data);
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

    let (data, extension, content_type) = if guessed_format == Some(ImageFormat::Gif) {
        (file_data.to_vec(), "gif", "image/gif")
    } else {
        let img = ImageReader::new(Cursor::new(&file_data))
            .with_guessed_format()
            .map_err(|e| {
                eprintln!("{}:{} - {}", file!(), line!(), e);
                (StatusCode::BAD_REQUEST, "Invalid format".to_string())
            })?
            .decode()
            .map_err(internal_error)?;

        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();

        let webp_data = Encoder::from_rgba(&rgba, width, height)
            .encode(compression_quality)
            .to_vec();

        (webp_data, "webp", "image/webp")
    };

    let timestamp = Utc::now().format("%Y-%m-%d_%H:%M:%S%.3f").to_string();
    let unique_filename = format!("{}.{}", timestamp, extension);

    let image_url = format!("{}/{}", bucket_endpoint, unique_filename);

    let bucket_client = create_bucket_client().await.map_err(internal_error)?;

    let put_result = bucket_client
        .put_object()
        .bucket(&bucket_name)
        .key(&unique_filename)
        .body(ByteStream::from(data))
        .content_type(content_type)
        .send()
        .await;

    match put_result {
        Ok(_) => {
            sqlx::query("INSERT INTO memes (image_url) VALUES ($1)")
                .bind(&image_url)
                .execute(&state.pool)
                .await
                .map_err(internal_error)?;

            Ok((StatusCode::CREATED, "Upload successful".to_string()))
        }
        Err(err) => {
            eprintln!("Upload failed: {:?}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Upload failed".to_string(),
            ))
        }
    }
}
