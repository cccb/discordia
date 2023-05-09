use anyhow::Result;
use sqlx::{sqlite::SqliteConnection, Connection as SqlConnection};
use std::sync::Arc;
use tokio::sync::Mutex;

/// A thread safe connection to the database
pub type Connection = Arc<Mutex<SqliteConnection>>;

/// Open a connection to the database
pub async fn open(filename: &str) -> Result<Connection> {
    let conn = SqliteConnection::connect(filename).await?;
    let conn = Arc::new(Mutex::new(conn));
    Ok(conn)
}
