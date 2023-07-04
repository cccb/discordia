use std::fs;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::ops::Deref;

use anyhow::Result;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteConnection},
    Connection as SqlConnection,
};
use tokio::sync::Mutex;

use crate::schema;

/// A thread safe connection to the database
pub struct Connection {
    filename: String,
    conn: Arc<Mutex<SqliteConnection>>,
    test: bool,
}

impl Deref for Connection {
    type Target = Mutex<SqliteConnection>;
    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        if !self.test {
            return;
        }
        let path = Path::new(&self.filename);
        if path.exists() {
            fs::remove_file(path).unwrap();
        }
    }
}

impl Connection {
    /// Open a connection to the database
    pub async fn open(filename: &str) -> Result<Self> {
        let conn = SqliteConnectOptions::from_str(filename)?.foreign_keys(true);
        let conn = SqliteConnection::connect_with(&conn).await?;
        let conn = Connection{
            filename: filename.to_string(),
            conn: Arc::new(Mutex::new(conn)),
            test: false,
        };
        Ok(conn)
    }

    /// Open a new test database connection.
    /// The database will be created on each open.
    pub async fn open_test() -> Self {
        let filename = format!(
            "/tmp/discordia_test_{}.sqlite3",
            rand::random::<u64>());

        let conn = SqliteConnectOptions::from_str(&filename)
            .unwrap()
            .create_if_missing(true)
            .foreign_keys(true);
        let conn = SqliteConnection::connect_with(&conn).await.unwrap();
        let conn = Connection {
            filename: filename.clone(),
            conn: Arc::new(Mutex::new(conn)),
            test: true,
        };

        // Install the schema
        schema::install(&conn).await.unwrap();

        conn
    }
}

