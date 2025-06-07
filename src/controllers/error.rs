use crate::AppState;
use axum::{
    extract::State,
    response::{Html, Redirect},
    routing::get,
    Router,
};
use std::sync::Arc;
use tera::Context;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", get(html))
}

async fn html(State(state): State<Arc<AppState>>) -> Result<Html<String>, Redirect> {
    let context = Context::new();

    let rendered = state
        .tera
        .render("error.html", &context)
        .unwrap_or_else(|_| "Internal server error".to_string());

    Ok(Html(rendered))
}
