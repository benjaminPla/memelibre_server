use memelibre;
use sqlx::{Executor, PgPool};
use std::env;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = PgPool::connect(&env::var("DB_CONN").expect("Missing DB_CONN env var")).await?;

    // pool.execute("DROP TABLE IF EXISTS memes;").await?;
    pool.execute("DROP TABLE IF EXISTS users;").await?;
    pool.execute("DROP TABLE IF EXISTS likes;").await?;

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
            id SERIAL PRIMARY KEY,
            is_admin BOOLEAN NOT NULL DEFAULT FALSE,
            username VARCHAR(32) UNIQUE NOT NULL
        );
        "#,
    )
    .await?;

    pool.execute(
        r#"
        CREATE TABLE likes (
            meme_id INTEGER REFERENCES memes(id),
            user_id INTEGER REFERENCES users(id),
            PRIMARY KEY (user_id, meme_id)
        );
        "#,
    )
    .await?;
    // SELECT m.id, m.title, m.image_url, COUNT(l.user_id) as like_count
    // FROM memes m
    // LEFT JOIN likes l ON m.id = l.meme_id
    // WHERE m.id = $1
    // GROUP BY m.id;

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
