use anyhow::Result;
use sqlx::Executor;

use crate::Connection;

/// Install the database schema.
pub async fn install(conn: &Connection) -> Result<()> {
    let mut conn = conn.lock().await;
    let schema_data = include_str!("../db/schema.sql");
    println!("installing database schema");
    (*conn).execute(schema_data).await?;
    Ok(())
}
