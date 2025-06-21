use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use std::env;

#[derive(Clone)]
#[allow(dead_code)]
pub struct Config {
    pub bucket_endpoint: String,
    pub bucket_key: String,
    pub bucket_name: String,
    pub bucket_object_max_size: usize,
    pub bucket_region: String,
    pub bucket_secret: String,
    pub client_url: String,
    pub compression_quality: f32,
    pub db_conn_string: String,
    pub db_max_conn: u32,
    pub jwt_secret: String,
    pub memes_pull_limit: i64,
    pub oauth_google_client_id: String,
    pub oauth_google_client_secret: String,
    pub oauth_redirect_uri: String,
    pub timeout_duration: u64,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        fn get_env_var(name: &str) -> Result<String, String> {
            env::var(name).map_err(|_| format!("Missing env var: {}", name))
        }

        fn get_and_parse_env_var<T: std::str::FromStr>(name: &str) -> Result<T, String>
        where
            T::Err: std::fmt::Display,
        {
            let value = get_env_var(name)?;
            value
                .parse()
                .map_err(|e| format!("Failed to parse {} env var: {}", name, e))
        }

        Ok(Self {
            bucket_endpoint: get_env_var("BUCKET_ENDPOINT")?,
            bucket_key: get_env_var("BUCKET_KEY")?,
            bucket_name: get_env_var("BUCKET_NAME")?,
            bucket_object_max_size: get_and_parse_env_var("BUCKET_OBJECT_MAX_SIZE")?,
            bucket_region: get_env_var("BUCKET_REGION")?,
            bucket_secret: get_env_var("BUCKET_SECRET")?,
            client_url: get_env_var("CLIENT_URL")?,
            compression_quality: get_and_parse_env_var::<f32>("COMPRESSION_QUALITY")?
                .clamp(0.0, 100.0),
            db_conn_string: get_env_var("DB_CONN_STRING")?,
            db_max_conn: get_and_parse_env_var("DB_MAX_CONN")?,
            jwt_secret: get_env_var("JWT_SECRET")?,
            memes_pull_limit: get_and_parse_env_var("MEMES_PULL_LIMIT")?,
            oauth_google_client_id: get_env_var("OATH_GOOGLE_CLIENT_ID")?,
            oauth_google_client_secret: get_env_var("OATH_GOOGLE_CLIENT_SECRET")?,
            oauth_redirect_uri: get_env_var("OAUTH_REDIRECT_URI")?,
            timeout_duration: get_and_parse_env_var("TIMEOUT_DURATION")?,
        })
    }
}

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: PgPool,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Meme {
    pub id: i32,
    pub image_url: String,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub is_admin: bool,
    pub username: String,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Like {
    pub meme_id: i32,
    pub user_id: String,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct JWTClaims {
    pub exp: usize,
    pub is_admin: bool,
    pub sub: String,
    pub username: String,
}

#[derive(Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
    pub token_type: String,
}

#[derive(Deserialize)]
pub struct Pagination {
    pub offset: Option<i32>,
}
