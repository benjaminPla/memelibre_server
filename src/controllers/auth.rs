use crate::AppState;
use axum::{extract::State, response::Redirect};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use rand::distr::Alphanumeric;
use rand::{rng, Rng};
use std::sync::Arc;

fn generate_state() -> String {
    rng()
        .sample_iter(Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

pub async fn handler(State(state): State<Arc<AppState>>, jar: CookieJar) -> (CookieJar, Redirect) {
    let oauth_state = generate_state();
    let redirect_uri = &state.config.oauth_redirect_uri;
    let scope = "openid";

    // Save `state` to cookie (or Redis, DB, session etc.)
    let cookie = Cookie::build(("oauth_state", oauth_state.clone()))
        .http_only(true)
        .max_age(cookie::time::Duration::minutes(5))
        .path("/")
        .same_site(SameSite::Lax)
        .secure(true)
        .build();

    let updated_jar = jar.add(cookie);

    let auth_url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope={}&access_type=offline&prompt=consent&state={}",
        &state.config.oauth_google_client_id,
        redirect_uri,
        scope,
        oauth_state
    );

    (updated_jar, Redirect::temporary(&auth_url))
}
