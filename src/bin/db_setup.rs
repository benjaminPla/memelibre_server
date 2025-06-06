use sqlx::{Executor, PgPool};
use std::env;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool =
        PgPool::connect(&env::var("DB_CONN_STRING").expect("Missing DB_CONN_STRING env var"))
            .await?;

    // pool.execute("DROP TABLE IF EXISTS memes;").await?;

    pool.execute(
        r#"
        CREATE TABLE IF NOT EXISTS memes (
            id SERIAL PRIMARY KEY,
            image_url TEXT NOT NULL
        );
        "#,
    )
    .await?;

    println!("Database setup complete.");

    Ok(())
}
