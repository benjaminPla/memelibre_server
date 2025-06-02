use memelibre;
use sqlx::{Executor, PgPool};
use std::env;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = PgPool::connect(&env::var("DB_CONN").expect("Missing DB_CONN env var")).await?;

    // pool.execute("DROP TABLE IF EXISTS memes;").await?;
    pool.execute("DROP TABLE IF EXISTS users;").await?;

    pool.execute(
        r#"
        CREATE TABLE IF NOT EXISTS memes (
            created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
            id SERIAL PRIMARY KEY,
            image_url TEXT NOT NULL
        );
        "#,
    )
    .await?;

    pool.execute(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
            hashed_password VARCHAR(255) NOT NULL,
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            is_admin BOOLEAN NOT NULL DEFAULT FALSE,
            username VARCHAR(32) UNIQUE NOT NULL
        );
        "#,
    )
    .await?;

    let hashed_password = memelibre::hash_password("12345");
    sqlx::query(
        r#"
        INSERT INTO users (hashed_password, is_admin, username)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(&hashed_password)
    .bind(true)
    .bind("admin")
    .execute(&pool)
    .await?;

    println!("Database setup complete.");

    Ok(())
}
