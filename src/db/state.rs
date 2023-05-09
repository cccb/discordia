use anyhow::Result;
use chrono::NaiveDate;
use sqlx::{FromRow, QueryBuilder};

use crate::database::Database;

#[derive(Debug, Clone, FromRow)]
pub struct State {
    pub accounts_calculated_at: NaiveDate,
}

impl State {
    /// Fetch current state from database
    pub async fn fetch(db: &Database) -> Result<Self> {
        let mut conn = db.lock().await;
        let state: State = sqlx::query_as("SELECT accounts_calculated_at FROM state")
            .fetch_one(&mut *conn)
            .await?;
        Ok(state)
    }

    pub async fn update(&self, db: &Database) -> Result<Self> {
        {
            let mut conn = db.lock().await;
            QueryBuilder::new("UPDATE state SET")
                .push(" accounts_calculated_at = ")
                .push_bind(self.accounts_calculated_at)
                .build()
                .execute(&mut *conn)
                .await?;
        }
        Self::fetch(db).await
    }
}
