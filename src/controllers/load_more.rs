use axum::{
    extract::{Path, State},
    response::Html,
    routing::get,
    Router,
};
use serde::Serialize;
use std::env;
use std::sync::Arc;
use tera::Context;

use crate::AppState;

#[derive(Serialize, sqlx::FromRow)]
struct Meme {
    id: i32,
    image_url: String,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/{last_id}", get(handler))
}

async fn handler(State(state): State<Arc<AppState>>, Path(last_id): Path<i32>) -> Html<String> {
    let limit = env::var("MEMES_PULL_LIMIT")
        .expect("Missing MEMES_PULL_LIMIT env var")
        .parse::<i64>()
        .expect("Error parsing MEMES_PULL_LIMIT env var");

    let memes: Vec<Meme> = sqlx::query_as(
        "
        SELECT id, image_url FROM memes
        WHERE id < $1
        ORDER BY id DESC
        LIMIT $2
        ",
    )
    .bind(last_id)
    .bind(limit)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let mut html_output = String::new();

    for meme in memes {
        let mut context = Context::new();
        context.insert("meme", &meme);

        let rendered = state
            .tera
            .render("_meme.html", &context)
            .unwrap_or_else(|e| {
                eprintln!("Tera rendering error: {}", e);
                "<!-- Error rendering meme -->".to_string()
            });

        html_output.push_str(&rendered);
    }

    Html(html_output)
}
