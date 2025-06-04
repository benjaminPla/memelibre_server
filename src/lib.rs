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
struct B2Response {
    authorizationToken: String,
    uploadUrl: String,
}

pub async fn get_b2_token() -> Result<B2Credentials, B2Error> {
    let bucket_id =
        env::var("B2_BUCKET_ID").map_err(|_| B2Error::EnvVarMissing("B2_BUCKET_ID".to_string()))?;

    let url = format!(
        "https://api003.backblazeb2.com/b2api/v4/b2_get_upload_url?bucketId={}",
        bucket_id
    );

    let client = Client::new();
    let response = client
        .get(&url)
        .header(
            "Authorization",
            "4_003fc32665896c00000000001_01bcc87d_864cdf_acct_fXpqkhRt0votDWZ1ogvu8xEjTWk=",
        )
        .send()
        .await
        .map_err(|e| B2Error::RequestFailed(e.to_string()))?;

    if !response.status().is_success() {
        return Err(B2Error::RequestFailed(format!(
            "B2 API returned status: {}",
            response.status()
        )));
    }

    let data: B2Response = response
        .json()
        .await
        .map_err(|e| B2Error::ParseFailed(e.to_string()))?;

    Ok(B2Credentials {
        auth_token: data.authorizationToken,
        upload_url: data.uploadUrl,
    })
}
