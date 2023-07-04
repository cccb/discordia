use anyhow::Result;
use sqlx::QueryBuilder;
use async_trait::async_trait;

use eris_domain::{Retrieve, State, Update};

use crate::Connection;

#[async_trait]
impl Retrieve<State> for Connection {
    type Filter = Option<bool>;

    /// Fetch current state from database
    async fn retrieve(&self, _filter: &Self::Filter) -> Result<State> {
        let mut conn = self.lock().await;
        let state: State = sqlx::query_as(
            "SELECT accounts_calculated_at FROM state")
            .fetch_one(&mut *conn)
            .await?;
        Ok(state)
    }
}

#[async_trait]
impl Update<State> for Connection {
    /// Update state in database
    async fn update(&self, state: State) -> Result<State> {
        {
            let mut conn = self.lock().await;
            QueryBuilder::new("UPDATE state SET")
                .push(" accounts_calculated_at = ")
                .push_bind(state.accounts_calculated_at)
                .build()
                .execute(&mut *conn)
                .await?;
        }
        self.retrieve(&None).await
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;

    #[tokio::test]
    async fn test_state_update_and_fetch() {
        let db = Connection::open_test().await;
        let mut state: State = db.retrieve(&None).await.unwrap();

        // Update state
        state.accounts_calculated_at = NaiveDate::from_ymd_opt(2023, 4, 2).unwrap();
        let state = db.update(state).await.unwrap();

        assert_eq!(
            state.accounts_calculated_at,
            NaiveDate::from_ymd_opt(2023, 4, 2).unwrap()
        );
    }
}
