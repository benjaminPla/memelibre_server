use axum::{
    extract::{Path, State},
    response::Html,
    routing::get,
    Router,
};
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
    Router::new().route("/{id}", get(handler))
}

async fn handler(State(state): State<Arc<AppState>>, Path(id): Path<i32>) -> Html<String> {
    let meme: Option<Meme> = sqlx::query_as("SELECT id, image_url FROM memes WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .unwrap_or(None);

    let mut context = Context::new();
    context.insert("meme", &meme);

    let rendered = state
        .tera
        .render("meme.html", &context)
        .unwrap_or_else(|e| format!("Template error: {}", e));

    Html(rendered)
}
