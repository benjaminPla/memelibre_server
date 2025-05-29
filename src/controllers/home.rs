use axum::{
    extract::{Query, State},
    response::Html,
    routing::get,
    Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tera::Context;

use crate::AppState;

#[derive(Serialize, sqlx::FromRow)]
struct Meme {
    title: String,
    image_url: String,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct Pagination {
    after: Option<DateTime<Utc>>,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", get(home))
}

async fn home(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<Pagination>,
) -> Html<String> {
    let limit = 100;

    let memes = if let Some(after) = pagination.after {
        sqlx::query_as::<_, Meme>(
            "SELECT title, image_url, created_at FROM memes
             WHERE created_at < $1
             ORDER BY created_at DESC
             LIMIT $2",
        )
        .bind(after)
        .bind(limit as i64)
        .fetch_all(&state.pool)
        .await
        .unwrap_or_else(|_| vec![])
    } else {
        sqlx::query_as::<_, Meme>(
            "SELECT title, image_url, created_at FROM memes
             ORDER BY created_at DESC
             LIMIT $1",
        )
        .bind(limit as i64)
        .fetch_all(&state.pool)
        .await
        .unwrap_or_else(|_| vec![])
    };

    let prev_cursor = memes.first().map(|m| m.created_at.to_rfc3339());
    let next_cursor = memes.last().map(|m| m.created_at.to_rfc3339());

    let mut context = Context::new();
    context.insert("memes", &memes);
    context.insert("prev_cursor", &prev_cursor);
    context.insert("next_cursor", &next_cursor);

    let rendered = state
        .tera
        .render("home.html", &context)
        .unwrap_or_else(|e| format!("Template error: {}", e));

    Html(rendered)
}
