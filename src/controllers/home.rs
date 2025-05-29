use axum::{extract::State, response::Html, routing::get, Router};
use serde::Serialize;
use std::sync::Arc;
use tera::{Context, Tera};

#[derive(Serialize)]
struct Meme {
    title: String,
    image_url: String,
}

pub fn router() -> Router<Arc<Tera>> {
    Router::new().route("/", get(home))
}

async fn home(State(tera): State<Arc<Tera>>) -> Html<String> {
    let memes = vec![
        Meme {
            title: "Distracted Boyfriend".into(),
            image_url: "https://i.imgflip.com/1ur9b0.jpg".into(),
        },
        Meme {
            title: "Drake Hotline Bling".into(),
            image_url: "https://i.imgflip.com/30b1gx.jpg".into(),
        },
        Meme {
            title: "Two Buttons".into(),
            image_url: "https://i.imgflip.com/1g8my4.jpg".into(),
        },
        Meme {
            title: "Change My Mind".into(),
            image_url: "https://i.imgflip.com/24y43o.jpg".into(),
        },
        Meme {
            title: "Left Exit 12 Off Ramp".into(),
            image_url: "https://i.imgflip.com/22bdq6.jpg".into(),
        },
    ];

    let mut context = Context::new();
    context.insert("memes", &memes);

    let rendered = tera
        .render("home.html", &context)
        .unwrap_or_else(|e| format!("Template error: {}", e));

    Html(rendered)
}
