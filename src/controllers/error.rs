use crate::AppState;
use axum::{extract::State, response::Html, routing::get, Router};
use std::sync::Arc;
use tera::Context;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", get(html))
}

async fn html(State(state): State<Arc<AppState>>) -> Result<Html<String>, String> {
    let context = Context::new();

    let rendered = state.tera.render("error.html", &context).map_err(|e| {
        eprintln!("{}:{} - {}", file!(), line!(), e);
        "<html><body><h1>Error</h1><p>Uy! algo salió mal</p></body></html>".to_string()
    })?;

    Ok(Html(rendered))
}
