use anyhow::Result;

use crate::db::Connection;

/// Install the database schema.
pub async fn install(conn: &Connection) -> Result<()> {
    let conn = conn.lock().await;
    let schema_data = include_str!("../../db/schema.sql");
    println!("schema_data: {}", schema_data);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Database;

    #[tokio::test]
    async fn test_install() {
        let conn = ::new().await;
        install(&conn).await.unwrap();
    }
}
