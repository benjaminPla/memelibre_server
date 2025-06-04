use axum::{extract::State, response::Html, routing::get, Router};
use serde::Serialize;
use std::sync::Arc;
use tera::Context;

use crate::AppState;

#[derive(Serialize, sqlx::FromRow)]
struct Meme {
    id: i32,
    image_url: String,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", get(handler))
}

async fn handler(State(state): State<Arc<AppState>>) -> Html<String> {
    let memes: Vec<Meme> = sqlx::query_as("SELECT image_url, id FROM memes")
        .fetch_all(&state.pool)
        .await
        .unwrap_or_else(|_| vec![]);

    let mut context = Context::new();
    context.insert("memes", &memes);

    let rendered = state
        .tera
        .render("home.html", &context)
        .unwrap_or_else(|e| format!("Template error: {}", e));

    Html(rendered)
}
