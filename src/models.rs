use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Deserialize, Serialize)]
pub struct JWTClaims {
    pub exp: usize,
    pub is_admin: bool,
    pub sub: Uuid,
    pub username: String,
}

#[derive(Deserialize)]
pub struct B2Credentials {
    pub auth_token: String,
    pub upload_url: String,
}
