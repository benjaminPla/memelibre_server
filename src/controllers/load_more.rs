use crate::http_error;
use crate::models::Meme;
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Html,
};
use std::sync::Arc;
use tera::{Context, Tera};

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Path(last_id): Path<i32>,
) -> Result<Html<String>, (StatusCode, String)> {
    let tera = Tera::new("src/templates/**/*")
        .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?;

    let memes: Vec<Meme> = sqlx::query_as(
        "
        SELECT id, image_url FROM memes
        WHERE id < $1
        ORDER BY id DESC
        LIMIT $2
        ",
    )
    .bind(last_id)
    .bind(&state.config.memes_pull_limit)
    .fetch_all(&state.db)
    .await
    .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?;

    let mut memes_html = String::new();

    for meme in &memes {
        let mut context = Context::new();
        context.insert("meme", meme);

        match tera.render("_meme.html", &context) {
            Ok(rendered) => memes_html.push_str(&rendered),
            Err(e) => {
                eprintln!("Failed to render meme: {}", e);
                continue;
            }
        }
    }

    Ok(Html(memes_html))
}
