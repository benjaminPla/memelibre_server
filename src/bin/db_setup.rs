use sqlx::{Executor, PgPool};

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let database_url = "postgres://postgres:password@localhost/memelibre";
    let pool = PgPool::connect(database_url).await?;

    // pool.execute("DROP TABLE IF EXISTS memes;").await?;

    pool.execute(
        r#"
        CREATE TABLE IF NOT EXISTS memes (
            id SERIAL PRIMARY KEY,
            title TEXT NOT NULL UNIQUE,
            image_url TEXT NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT now()
        );
        "#,
    )
    .await?;

    pool.execute(
        r#"
        INSERT INTO memes (title, image_url)
        VALUES
            ('Distracted Boyfriend', 'https://i.imgflip.com/1ur9b0.jpg'),
            ('Drake Hotline Bling', 'https://i.imgflip.com/30b1gx.jpg'),
            ('Two Buttons', 'https://i.imgflip.com/1g8my4.jpg'),
            ('Change My Mind', 'https://i.imgflip.com/24y43o.jpg'),
            ('Left Exit 12 Off Ramp', 'https://i.imgflip.com/22bdq6.jpg')
        ON CONFLICT (title) DO NOTHING;
        "#,
    )
    .await?;

    println!("Database setup complete.");

    Ok(())
}
