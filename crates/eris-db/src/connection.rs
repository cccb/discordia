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
use async_trait::async_trait;

use crate::schema;


/// A thread safe connection to the database
pub type Connection = Arc<Mutex<SqliteConnection>>;


#[async_trait]
pub trait Query<F, T> {
    async fn query(&self, filter: &F) -> Result<Vec<T>>;
}

#[async_trait]
pub trait Insert<T> {
    async fn insert(&self, item: T) -> Result<T>;
}

#[async_trait]
pub trait Retrieve<F, T> {
    async fn retrieve(&self, filter: &F) -> Result<T>;
}

#[async_trait]
pub trait Delete<T> {
    async fn delete(&self, item: T) -> Result<()>;
}


/// Open a connection to the database
pub async fn open(filename: &str) -> Result<Connection> {
    let conn = SqliteConnectOptions::from_str(filename)?
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

    let conn = SqliteConnectOptions::from_str(&filename).unwrap()
        .create_if_missing(true)
        .foreign_keys(true);
    let conn = SqliteConnection::connect_with(&conn).await.unwrap();
    let conn = Arc::new(Mutex::new(conn));

    // Install the schema
    schema::install(&conn).await.unwrap();

    (handle, conn)
}
