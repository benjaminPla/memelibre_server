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
            email VARCHAR(255) UNIQUE NOT NULL,
            hashed_password VARCHAR(255) NOT NULL,
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            is_admin BOOLEAN NOT NULL DEFAULT FALSE,
            username VARCHAR(32) UNIQUE NOT NULL
        );
        "#,
    )
    .await?;

    // pool.execute(
    // r#"
    // INSERT INTO memes (image_url)
    // VALUES
    // ('https://i.imgflip.com/1ur9b0.jpg'),
    // ('https://i.imgflip.com/30b1gx.jpg'),
    // ('https://i.imgflip.com/1g8my4.jpg'),
    // ('https://i.imgflip.com/24y43o.jpg'),
    // ('https://i.imgflip.com/22bdq6.jpg');
    // "#,
    // )
    // .await?;

    pool.execute(
        r#"
        INSERT INTO users (email,hashed_password, is_admin, username)
        VALUES ('benjaminpla.dev@gmail.com', '12345', true, 'ben')
        "#,
    )
    .await?;

    println!("Database setup complete.");

    Ok(())
}
