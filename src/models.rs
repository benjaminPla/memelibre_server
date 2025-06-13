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
    pub compression_quality: f32,
    pub db_conn_string: String,
    pub db_max_conn: u32,
    pub memes_pull_limit: i64,
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
            compression_quality: get_and_parse_env_var::<f32>("COMPRESSION_QUALITY")?
                .clamp(0.0, 100.0),
            db_conn_string: get_env_var("DB_CONN_STRING")?,
            db_max_conn: get_and_parse_env_var("DB_MAX_CONN")?,
            memes_pull_limit: get_and_parse_env_var("MEMES_PULL_LIMIT")?,
            timeout_duration: get_and_parse_env_var("TIMEOUT_DURATION")?,
        })
    }
}
