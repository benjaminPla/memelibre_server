use crate::http_error;
use crate::models;
use aws_sdk_s3::primitives::ByteStream;
use axum::{
    extract::{Extension, Multipart, State},
    http::status::StatusCode,
};
use chrono::Utc;
use image::{ImageFormat, ImageReader};
use memelibre_server::create_bucket_client;
use std::io::Cursor;
use std::sync::Arc;
use webp::Encoder;

pub async fn handler(
    State(state): State<Arc<models::AppState>>,
    Extension(claims): Extension<models::JWTClaims>,
    mut multipart: Multipart,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let mut file_data: Option<bytes::Bytes> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?
    {
        if field.name() == Some("file") {
            let data = field
                .bytes()
                .await
                .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?;

            if data.len() > state.config.bucket_object_max_size {
                return Err(http_error!(StatusCode::PAYLOAD_TOO_LARGE));
            }
            file_data = Some(data);
        }
    }

    let file_data = file_data.ok_or((StatusCode::BAD_REQUEST, "File is empty".to_string()))?;

    let guessed_format = ImageReader::new(Cursor::new(&file_data))
        .with_guessed_format()
        .map_err(|_| http_error!(StatusCode::BAD_REQUEST, "Invalid image format"))?
        .format();

    let (data, extension, content_type) = if guessed_format == Some(ImageFormat::Gif) {
        (file_data.to_vec(), "gif", "image/gif")
    } else {
        let img = ImageReader::new(Cursor::new(&file_data))
            .with_guessed_format()
            .map_err(|_| http_error!(StatusCode::BAD_REQUEST, "Invalid image format"))?
            .decode()
            .map_err(|_| http_error!(StatusCode::BAD_REQUEST, "Invalid image format"))?;

        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();

        let webp_data = Encoder::from_rgba(&rgba, width, height)
            .encode(state.config.compression_quality)
            .to_vec();

        (webp_data, "webp", "image/webp")
    };

    let timestamp = Utc::now().format("%Y-%m-%d_%H:%M:%S%.3f").to_string();
    let unique_filename = format!("{}.{}", timestamp, extension);

    let image_url = format!(
        "https://{}.{}/{}",
        &state.config.bucket_name,
        &state
            .config
            .bucket_endpoint
            .strip_prefix("https://")
            .ok_or_else(|| http_error!(
                StatusCode::INTERNAL_SERVER_ERROR,
                "BUCKET_ENDPOINT env var missing https:// prefix"
            ))?,
        unique_filename
    );

    let bucket_client = create_bucket_client()
        .await
        .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?;

    let put_result = bucket_client
        .put_object()
        .bucket(&state.config.bucket_name)
        .key(&unique_filename)
        .body(ByteStream::from(data))
        .content_type(content_type)
        .acl("public-read".into())
        .send()
        .await;

    match put_result {
        Ok(_) => {
            sqlx::query("INSERT INTO memes (created_by, image_url, like_count) VALUES ($1, $2, 0)")
                .bind(&claims.sub)
                .bind(&image_url)
                .execute(&state.db)
                .await
                .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?;

            Ok((StatusCode::CREATED, "Upload successful".to_string()))
        }
        Err(err) => Err(http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: err)),
    }
}
