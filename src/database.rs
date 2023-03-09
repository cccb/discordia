use anyhow::Result;
use sqlx::{sqlite::SqliteConnection, Connection};
use std::sync::Arc;
use tokio::sync::Mutex;

pub type Database = Arc<Mutex<SqliteConnection>>;

pub async fn connect(filename: &str) -> Result<Database> {
    let conn = SqliteConnection::connect(filename).await?;
    let db = Arc::new(Mutex::new(conn));

    Ok(db)
}
