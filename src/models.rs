use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JWTClaims {
    pub exp: usize,
    pub is_admin: bool,
    pub sub: i32,
    pub username: String,
}

#[derive(Deserialize)]
pub struct B2Credentials {
    pub auth_token: String,
    pub upload_url: String,
}
