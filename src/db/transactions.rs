use anyhow::Result;
use chrono::NaiveDate;
use sqlx::{FromRow, QueryBuilder, Sqlite};

use crate::database::Database;
use crate::db::{errors::Error, results::Insert};

#[derive(Debug, Default, Clone)]
pub struct TransactionFilter {
    pub member_id: Option<u32>,
    pub date: Option<NaiveDate>,
    pub date_before: Option<NaiveDate>,
    pub date_after: Option<NaiveDate>,
}

#[derive(Debug, Default, Clone, FromRow)]
pub struct Transaction {
    pub id: u32,
    pub member_id: u32,
    pub date: NaiveDate,
    pub account_name: String,
    pub amount: f64,
    pub description: String,
}

impl Transaction {
    // Filter transactions
    pub async fn filter(
        db: &Database,
        filter: Option<TransactionFilter>,
    ) -> Result<Vec<Transaction>> {
        let mut conn = db.lock().await;
        let mut qry = QueryBuilder::<Sqlite>::new(
            r#"
            SELECT 
                id,
                member_id,
                date,
                account_name,
                ROUND(amount, 10) AS amount,
                description
            FROM transactions
            WHERE 1
            "#,
        );
        if let Some(filter) = filter {
            if let Some(member_id) = filter.member_id {
                qry.push(" AND member_id = ").push_bind(member_id);
            }
            if let Some(date) = filter.date {
                qry.push(" AND date = ").push_bind(date);
            }
            if let Some(date_before) = filter.date_before {
                qry.push(" AND date <= ").push_bind(date_before);
            }
            if let Some(date_after) = filter.date_after {
                qry.push(" AND date >= ").push_bind(date_after);
            }
        }
        let transactions: Vec<Transaction> = qry.build_query_as().fetch_all(&mut *conn).await?;
        Ok(transactions)
    }

    /// Fetch a single transaction by ID
    pub async fn get(db: &Database, id: u32) -> Result<Transaction> {
        let filter = TransactionFilter {
            member_id: Some(id),
            ..TransactionFilter::default()
        };
        let transaction: Transaction = Self::filter(db, Some(filter))
            .await?
            .pop()
            .ok_or_else(|| Error::NotFound)?;
        Ok(transaction)
    }

    /// Update a transaction
    pub async fn update(&self, db: &Database) -> Result<Transaction> {
        {
            let mut conn = db.lock().await;
            QueryBuilder::<Sqlite>::new("UPDATE transactions SET")
                .push(" member_id = ")
                .push_bind(self.member_id)
                .push(", date = ")
                .push_bind(self.date)
                .push(", account_name = ")
                .push_bind(&self.account_name)
                .push(", amount = ")
                .push_bind(self.amount)
                .push(", description = ")
                .push_bind(&self.description)
                .push(" WHERE id = ")
                .push_bind(self.id)
                .build()
                .execute(&mut *conn)
                .await?;
        }
        Self::get(db, self.id).await
    }

    /// Create transaction
    pub async fn create(&self, db: &Database) -> Result<Transaction> {
        let insert: Insert = {
            let mut conn = db.lock().await;
            let mut qry = QueryBuilder::<Sqlite>::new(
                r#"INSERT INTO transactions (
                    member_id,
                    date,
                    account_name,
                    amount,
                    description
                ) VALUES (
                "#,
            );
            qry.separated(", ")
                .push_bind(self.member_id)
                .push_bind(self.date)
                .push_bind(&self.account_name)
                .push_bind(self.amount)
                .push_bind(&self.description);

            qry.push(") RETURNING id ")
                .build_query_as()
                .fetch_one(&mut *conn)
                .await?
        };
        Self::get(db, insert.id).await
    }
}
