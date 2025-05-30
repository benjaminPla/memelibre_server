use sqlx::{Executor, PgPool};
use std::env;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = PgPool::connect(&env::var("DB_CONN").expect("Missing DB_CONN env var")).await?;

    // pool.execute("DROP TABLE IF EXISTS memes;").await?;

    pool.execute(
        r#"
        CREATE TABLE IF NOT EXISTS memes (
            id SERIAL PRIMARY KEY,
            image_url TEXT NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT now()
        );
        "#,
    )
    .await?;

    pool.execute(
        r#"
        INSERT INTO memes (image_url)
        VALUES
            ('https://i.imgflip.com/1ur9b0.jpg'),
            ('https://i.imgflip.com/30b1gx.jpg'),
            ('https://i.imgflip.com/1g8my4.jpg'),
            ('https://i.imgflip.com/24y43o.jpg'),
            ('https://i.imgflip.com/22bdq6.jpg');
        "#,
    )
    .await?;

    println!("Database setup complete.");

    Ok(())
}
