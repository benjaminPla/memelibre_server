use reqwest::Client;
use serde::Deserialize;
use std::env;
use thiserror::Error;

#[derive(Deserialize)]
pub struct B2Credentials {
    pub auth_token: String,
    pub upload_url: String,
}

#[derive(Error, Debug)]
pub enum B2Error {
    #[error("Environment variable missing: {0}")]
    EnvVarMissing(String),
    #[error("B2 API request failed: {0}")]
    RequestFailed(String),
    #[error("Failed to parse B2 response: {0}")]
    ParseFailed(String),
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthResponse {
    authorization_token: String,
    api_info: ApiInfo,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiInfo {
    storage_api: StorageApi,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StorageApi {
    api_url: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UploadUrlResponse {
    authorization_token: String,
    upload_url: String,
}

pub async fn get_b2_token() -> Result<B2Credentials, B2Error> {
    let b2_application_key = env::var("B2_APPLICATION_KEY")
        .map_err(|_| B2Error::EnvVarMissing("B2_APPLICATION_KEY".to_string()))?;
    let b2_authentication_url = env::var("B2_AUTHENTICATION_URL")
        .map_err(|_| B2Error::EnvVarMissing("B2_AUTHENTICATION_URL".to_string()))?;
    let b2_bucket_id =
        env::var("B2_BUCKET_ID").map_err(|_| B2Error::EnvVarMissing("B2_BUCKET_ID".to_string()))?;
    let b2_key_id =
        env::var("B2_KEY_ID").map_err(|_| B2Error::EnvVarMissing("B2_KEY_ID".to_string()))?;

    let client = Client::new();

    let auth_resp: AuthResponse = client
        .get(&b2_authentication_url)
        .basic_auth(b2_key_id, Some(b2_application_key))
        .send()
        .await
        .map_err(|e| B2Error::RequestFailed(format!("Auth request failed: {}", e)))?
        .json()
        .await
        .map_err(|e| B2Error::ParseFailed(format!("Auth response parse failed: {}", e)))?;

    let upload_url = format!(
        "{}/b2api/v4/b2_get_upload_url",
        auth_resp.api_info.storage_api.api_url
    );

    let upload_resp: UploadUrlResponse = client
        .post(&upload_url)
        .header("Authorization", &auth_resp.authorization_token)
        .json(&serde_json::json!({ "bucketId": b2_bucket_id }))
        .send()
        .await
        .map_err(|e| B2Error::RequestFailed(format!("Upload URL request failed: {}", e)))?
        .json()
        .await
        .map_err(|e| B2Error::ParseFailed(format!("Upload URL parse failed: {}", e)))?;

    Ok(B2Credentials {
        auth_token: upload_resp.authorization_token,
        upload_url: upload_resp.upload_url,
    })
}
