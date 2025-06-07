use axum::{
    extract::State,
    response::{Html, Redirect},
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
    Router::new().route("/", get(handler))
}

async fn handler(State(state): State<Arc<AppState>>) -> Result<Html<String>, Redirect> {
    let limit = env::var("MEMES_PULL_LIMIT")
        .map_err(|e| {
            eprintln!("{}:{} - {}", file!(), line!(), e);
            Redirect::to("/error")
        })?
        .parse::<i64>()
        .map_err(|e| {
            eprintln!("{}:{} - {}", file!(), line!(), e);
            Redirect::to("/error")
        })?;

    let memes: Vec<Meme> = sqlx::query_as(
        "
        SELECT id, image_url
        FROM memes
        ORDER BY id DESC
        LIMIT $1
        ",
    )
    .bind(limit)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let mut memes_html = String::new();

    for meme in &memes {
        let mut context = Context::new();
        context.insert("meme", meme);

        match state.tera.render("_meme.html", &context) {
            Ok(rendered) => memes_html.push_str(&rendered),
            Err(e) => {
                eprintln!("Failed to render meme: {}", e);
                continue;
            }
        }
    }

    let mut context = Context::new();
    context.insert("memes", &memes_html);

    let rendered = state.tera.render("home.html", &context).map_err(|e| {
        eprintln!("{}:{} - {}", file!(), line!(), e);
        Redirect::to("/error")
    })?;

    Ok(Html(rendered))
}
