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

use crate::schema;

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

pub struct TestHandle {
    filename: String
}

impl Drop for TestHandle {
    fn drop(&mut self) {
        let path = Path::new(&self.filename);
        if path.exists() {
            fs::remove_file(path).unwrap();
        }
    }
}


/// Open a new test database connection.
/// The database will be created on each open.
pub async fn open_test() -> (TestHandle, Connection) {
    let filename = format!("/tmp/discordia_test_{}.sqlite3", rand::random::<u64>());
    let handle = TestHandle { filename: filename.clone() };
    let conn = open(&filename).await.unwrap();

    // Install the schema
    schema::install(&conn).await.unwrap();

    (handle, conn)
}
