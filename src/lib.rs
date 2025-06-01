mod models;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use reqwest::Client;
use serde::Deserialize;
use std::env;

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

#[derive(Deserialize)]
struct B2Response {
    authorizationToken: String,
    uploadUrl: String,
}

pub async fn get_b2_token() -> Result<models::B2Credentials, reqwest::Error> {
    let client = Client::new();

    // here I need to get b2_authenticate

    let bucket_id = env::var("B2_BUCKET_ID").expect("Missing B2_BUCKET_ID env var");

    let url = format!(
        "https://api003.backblazeb2.com/b2api/v4/b2_get_upload_url?bucketId={}",
        bucket_id
    );

    let response = dbg!(client.get(url)
        .header("Authorization", "4_003fc32665896c00000000001_01bcc2b7_ab6d4e_acct_LHVLgZtwYcKAS_eysVDDkOpr5LY=")
        .send().await?);

    let data: B2Response = response.json().await?;

    Ok(models::B2Credentials {
        auth_token: data.authorizationToken,
        upload_url: data.uploadUrl,
    })
}
