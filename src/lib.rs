mod models;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use reqwest::Client;
use serde::Deserialize;
use std::env;
use thiserror::Error;

pub fn hash_password(password: &str) -> String {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let hashed_password = argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("Error on `hash_password`")
        .to_string();
    hashed_password
}

pub fn verify_password(hashed_password: &str, password: &str) -> bool {
    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(hashed_password).expect("Error parsing hashed password");
    argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
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

pub async fn get_b2_token() -> Result<models::B2Credentials, B2Error> {
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

    Ok(models::B2Credentials {
        auth_token: data.authorizationToken,
        upload_url: data.uploadUrl,
    })
}
