use anyhow::Result;
use chrono::NaiveDate;
use sqlx::{FromRow, QueryBuilder};

use crate::Connection;

#[derive(Debug, Clone, FromRow)]
pub struct State {
    pub accounts_calculated_at: NaiveDate,
}

impl State {
    /// Fetch current state from database
    pub async fn fetch(conn: &Connection) -> Result<Self> {
        let mut conn = conn.lock().await;
        let state: State = sqlx::query_as("SELECT accounts_calculated_at FROM state")
            .fetch_one(&mut *conn)
            .await?;
        Ok(state)
    }

    pub async fn update(&self, conn: &Connection) -> Result<Self> {
        {
            let mut conn = conn.lock().await;
            QueryBuilder::new("UPDATE state SET")
                .push(" accounts_calculated_at = ")
                .push_bind(self.accounts_calculated_at)
                .build()
                .execute(&mut *conn)
                .await?;
        }
        Self::fetch(conn).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::connection;

    #[tokio::test]
    async fn test_state_update_and_fetch() {
        let (_handle, conn) = connection::open_test().await;
        let mut state = State::fetch(&conn).await.unwrap();

        // Update state
        state.accounts_calculated_at = NaiveDate::from_ymd_opt(2023, 4, 2).unwrap();
        let state = state.update(&conn).await.unwrap();

        assert_eq!(
            state.accounts_calculated_at,
            NaiveDate::from_ymd_opt(2023, 4, 2).unwrap()
        );
    }
}
