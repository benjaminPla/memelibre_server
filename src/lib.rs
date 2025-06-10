use aws_config::SdkConfig;
use aws_credential_types::{provider::SharedCredentialsProvider, Credentials};
use aws_sdk_s3::{
    config::{BehaviorVersion, Region},
    Client,
};
use axum::http::status::StatusCode;
use std::env;

pub fn internal_error<E: std::fmt::Display>(err: E) -> (StatusCode, String) {
    eprintln!("{}:{} - {}", file!(), line!(), err);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Internal server error".to_string(),
    )
}

pub async fn create_bucket_client() -> Result<Client, String> {
    let bucket_endpoint =
        env::var("BUCKET_ENDPOINT").map_err(|e| format!("{}:{} - {}", file!(), line!(), e))?;
    let bucket_key =
        env::var("BUCKET_KEY").map_err(|e| format!("{}:{} - {}", file!(), line!(), e))?;
    let bucket_region =
        env::var("BUCKET_REGION").map_err(|e| format!("{}:{} - {}", file!(), line!(), e))?;
    let bucket_secret =
        env::var("BUCKET_SECRET").map_err(|e| format!("{}:{} - {}", file!(), line!(), e))?;

    let credentials = Credentials::new(bucket_key, bucket_secret, None, None, "digitalocean");

    let credentials_provider = SharedCredentialsProvider::new(credentials);

    let sdk_config = SdkConfig::builder()
        .region(Some(Region::new(bucket_region)))
        .endpoint_url(bucket_endpoint)
        .credentials_provider(credentials_provider)
        .behavior_version(BehaviorVersion::latest())
        .build();

    Ok(Client::new(&sdk_config))
}
