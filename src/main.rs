mod controllers;
mod macros;
mod middlewares;
mod models;
mod routes;

use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let config = models::Config::from_env().expect("Error creating Config");

    let db = PgPoolOptions::new()
        .max_connections(config.db_max_conn)
        .connect(&config.db_conn_string)
        .await
        .expect("Error connecting to database");

    let state = Arc::new(models::AppState { config, db });

    let app = routes::create_route(&state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Error binding to port 3000");

    axum::serve(listener, app)
        .await
        .expect("Error starting server");
}
