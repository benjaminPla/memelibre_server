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
    page: Option<u32>,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", get(home))
}

async fn home(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<Pagination>,
) -> Html<String> {
    let page = pagination.page.unwrap_or(1).max(1);
    let limit = 100;
    let offset = (page - 1) * limit;

    let memes = sqlx::query_as::<_, Meme>(
        "SELECT title, image_url, created_at FROM memes
    ORDER BY created_at DESC
    LIMIT $1 OFFSET $2",
    )
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_else(|_| vec![]);

    let mut context = Context::new();
    context.insert("memes", &memes);
    context.insert("page", &page);

    let rendered = state
        .tera
        .render("home.html", &context)
        .unwrap_or_else(|e| format!("Template error: {}", e));

    Html(rendered)
}
