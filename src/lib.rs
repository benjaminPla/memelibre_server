use aws_config::SdkConfig;
use aws_credential_types::{provider::SharedCredentialsProvider, Credentials};
use aws_sdk_s3::{
    config::{BehaviorVersion, Region},
    Client,
};
use axum::http::status::StatusCode;

mod models;

pub fn internal_error<E: std::fmt::Display>(err: E) -> (StatusCode, String) {
    eprintln!("{}:{} - {}", file!(), line!(), err);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Internal server error".to_string(),
    )
}

pub async fn create_bucket_client() -> Result<Client, String> {
    let config = models::Config::from_env().expect("Error creating Config");

    let credentials = Credentials::new(
        config.bucket_key,
        config.bucket_secret,
        None,
        None,
        "digitalocean",
    );

    let credentials_provider = SharedCredentialsProvider::new(credentials);

    let sdk_config = SdkConfig::builder()
        .region(Some(Region::new(config.bucket_region)))
        .endpoint_url(config.bucket_endpoint)
        .credentials_provider(credentials_provider)
        .behavior_version(BehaviorVersion::latest())
        .build();

    Ok(Client::new(&sdk_config))
}
