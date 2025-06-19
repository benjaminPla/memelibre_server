use crate::http_error;
use crate::models;
use crate::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Redirect,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use chrono;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct OAuthCallback {
    code: Option<String>,
    error: Option<String>,
    state: Option<String>,
}

#[derive(Deserialize)]
struct UserInfo {
    id: String,
}

async fn exchange_code_for_token(
    state: &AppState,
    client: &reqwest::Client,
    auth_code: &str,
) -> Result<models::TokenResponse, (StatusCode, String)> {
    let params = vec![
        ("client_id", state.config.oauth_google_client_id.as_str()),
        (
            "client_secret",
            state.config.oauth_google_client_secret.as_str(),
        ),
        ("code", auth_code),
        ("grant_type", "authorization_code"),
        ("redirect_uri", state.config.oauth_redirect_uri.as_str()),
    ];

    let response = client
        .post("https://oauth2.googleapis.com/token")
        .form(&params)
        .send()
        .await
        .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?;

    if !response.status().is_success() {
        let e = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e));
    }

    let token_response: models::TokenResponse = response
        .json()
        .await
        .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?;

    Ok(token_response)
}

async fn get_user_info(
    client: &reqwest::Client,
    access_token: &str,
) -> Result<UserInfo, (StatusCode, String)> {
    let response = client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?;

    if !response.status().is_success() {
        let e = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e));
    }

    let user_info: UserInfo = response
        .json()
        .await
        .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?;

    Ok(user_info)
}

async fn create_user_session(
    state: &AppState,
    user_info: &UserInfo,
) -> Result<String, (StatusCode, String)> {
    let existing_user: Option<models::User> =
        sqlx::query_as("SELECT id, is_admin, username FROM users WHERE id = $1")
            .bind(&user_info.id)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?;

    let user = match existing_user {
        Some(user) => user,
        None => {
            let mut generated_username = memelibre_server::generate_username()?;

            loop {
                let existing_username: Option<(String,)> =
                    sqlx::query_as("SELECT username FROM users WHERE username = $1")
                        .bind(&generated_username)
                        .fetch_optional(&state.db)
                        .await
                        .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?;

                if existing_username.is_none() {
                    break;
                }

                generated_username = memelibre_server::generate_username()?;
            }

            sqlx::query_as(
                "INSERT INTO users (id, is_admin, username) VALUES ($1, false, $2) RETURNING id, is_admin, username",
            )
            .bind(&user_info.id)
            .bind(&generated_username)
            .fetch_one(&state.db)
            .await
            .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?
        }
    };

    let claims = models::JWTClaims {
        exp: (chrono::Utc::now() + chrono::Duration::days(1)).timestamp() as usize,
        is_admin: user.is_admin,
        sub: user.id,
        username: user.username,
    };

    let session_token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(&state.config.jwt_secret.as_bytes()),
    )
    .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?;

    Ok(session_token)
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Query(params): Query<OAuthCallback>,
) -> Result<(CookieJar, Redirect), (StatusCode, String)> {
    let client = reqwest::Client::new();

    // Step 1: Verify state parameter (CSRF protection)
    let stored_state = jar
        .get("oauth_state")
        .ok_or(http_error!(StatusCode::UNAUTHORIZED))?
        .value();

    let received_state = params
        .state
        .as_ref()
        .ok_or(http_error!(StatusCode::UNAUTHORIZED))?;

    if stored_state != received_state {
        return Err(http_error!(StatusCode::UNAUTHORIZED));
    }

    // Step 2: Check for authorization errors
    if let Some(e) = params.error {
        return Err(http_error!(StatusCode::UNAUTHORIZED, err: e));
    }

    // Step 2c: Extract authorization code
    let auth_code = params.code.ok_or(http_error!(StatusCode::UNAUTHORIZED))?;

    // Step 3: Exchange authorization code for access token
    let token_response = exchange_code_for_token(&state, &client, &auth_code).await?;

    // Step 4: Use access token to get user info
    let user_info = get_user_info(&client, &token_response.access_token).await?;

    // Step 5: Create user session/JWT (implementation depends on your auth system)
    let session_token = create_user_session(&state, &user_info).await?;

    // Step 6: Set session cookie and redirect
    let session_cookie = Cookie::build(("session", session_token))
        .http_only(true)
        .max_age(cookie::time::Duration::days(15))
        .path("/")
        .same_site(SameSite::Lax)
        .secure(true)
        .build();

    let updated_jar = jar.add(session_cookie).remove(Cookie::from("oauth_state"));

    Ok((updated_jar, Redirect::to(&state.config.client_url)))
}
