use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;

use crate::error::{ErrorContext, Result};

pub async fn connect(database_path: &str) -> Result<SqlitePool> {
    let options = SqliteConnectOptions::new()
        .filename(database_path)
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .db_context("open connection")?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS pages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            url TEXT NOT NULL UNIQUE,
            title TEXT,
            body TEXT NOT NULL,
            fetched_at TEXT NOT NULL DEFAULT (datetime('now'))
        )
        "#,
    )
    .execute(&pool)
    .await
    .db_context("create schema")?;

    Ok(pool)
}

pub async fn save_page(
    pool: &SqlitePool,
    url: &str,
    title: Option<&str>,
    body: &str,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO pages (url, title, body)
        VALUES (?1, ?2, ?3)
        ON CONFLICT(url) DO UPDATE SET
            title = excluded.title,
            body = excluded.body,
            fetched_at = datetime('now')
        "#,
    )
    .bind(url)
    .bind(title)
    .bind(body)
    .execute(pool)
    .await
    .db_context("save page")?;

    Ok(())
}
