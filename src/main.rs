use axum::{
    http::HeaderValue,
    middleware,
    routing::{delete, get, post},
    Router,
};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::sync::Arc;
use std::time::Duration;
use tower_http::{
    compression::CompressionLayer,
    cors::{AllowOrigin, Any, CorsLayer},
    timeout::TimeoutLayer,
};

mod controllers;
mod macros;
mod middlewares;
mod models;

#[derive(Clone)]
pub struct AppState {
    config: models::Config,
    db: PgPool,
}

#[tokio::main]
async fn main() {
    let config = models::Config::from_env().expect("Error creating Config");
    let timeout_duration = config.timeout_duration;
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list([
            "https://memelibre.com".parse::<HeaderValue>().unwrap(),
            "http://memelibre.com".parse::<HeaderValue>().unwrap(),
        ]))
        .allow_methods(Any)
        .allow_headers(Any);

    let db = PgPoolOptions::new()
        .max_connections(config.db_max_conn)
        .connect(&config.db_conn_string)
        .await
        .expect("Error connecting to database");

    let app_state = Arc::new(AppState { config, db });

    // Auth routes
    let auth_routes = Router::new()
        .route("/", get(controllers::auth::handler))
        .route("/callback", get(controllers::auth_callback::handler));

    // Meme routes
    let meme_routes = Router::new()
        .route("/get", get(controllers::meme_get_all::handler))
        .route("/get/{id}", get(controllers::meme_get_by_id::handler))
        .route("/post", post(controllers::meme_post::handler))
        .route(
            "/delete/{id}",
            delete(controllers::delete::handler)
                .layer(middleware::from_fn_with_state(
                    app_state.clone(),
                    middlewares::with_is_admin::handler,
                ))
                .layer(middleware::from_fn_with_state(
                    app_state.clone(),
                    middlewares::with_auth::handler,
                )),
        );

    let app = Router::new()
        .nest(
            "/api",
            Router::new()
                .nest("/auth", auth_routes)
                .nest("/meme", meme_routes)
                .route("/load_more/{id}", get(controllers::load_more::handler))
                .with_state(app_state),
        )
        .layer(cors)
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(timeout_duration)));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Error binding to port 3000");

    axum::serve(listener, app)
        .await
        .expect("Error starting server");
}
