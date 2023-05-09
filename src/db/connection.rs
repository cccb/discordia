use std::fs;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::Result;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteConnection},
    Connection as SqlConnection,
};
use tokio::sync::Mutex;

use crate::db::schema;

/// A thread safe connection to the database
pub type Connection = Arc<Mutex<SqliteConnection>>;

/// Open a connection to the database
pub async fn open(filename: &str) -> Result<Connection> {
    let conn = SqliteConnectOptions::from_str(filename)?
        .create_if_missing(true)
        .foreign_keys(true);
    let conn = SqliteConnection::connect_with(&conn).await?;
    let conn = Arc::new(Mutex::new(conn));
    Ok(conn)
}

/// Open a new test database connection.
/// The database will be created on each open.
pub async fn open_test() -> Connection {
    let filename = format!("/tmp/discordia_test_{}.sqlite3", rand::random::<u64>());
    let conn = open(&filename).await.unwrap();

    // Install the schema
    schema::install(&conn).await.unwrap();

    conn
}
