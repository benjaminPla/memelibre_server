use crate::models;
use axum::{extract::State, response::Redirect};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use std::sync::Arc;

pub async fn handler(
    State(state): State<Arc<models::AppState>>,
    jar: CookieJar,
) -> (CookieJar, Redirect) {
    let session_cookie = Cookie::build(("session_token", ""))
        .http_only(true)
        .max_age(cookie::time::Duration::seconds(0))
        .path("/")
        .same_site(SameSite::Lax)
        .secure(true)
        .build();

    let updated_jar = jar.add(session_cookie);

    (updated_jar, Redirect::permanent(&state.config.client_url))
}
