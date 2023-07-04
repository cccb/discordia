use anyhow::Result;
use chrono::NaiveDate;
use sqlx::{FromRow, QueryBuilder, Sqlite};

use crate::{
    connection::Connection, 
    errors::Error,
    results::Insert,
};

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
        db: &Connection,
        filter: &TransactionFilter,
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
        if let Some(member_id) = filter.member_id {
            qry.push(" AND member_id = ").push_bind(member_id);
        }
        if let Some(date) = filter.date.clone() {
            qry.push(" AND date = ").push_bind(date);
        }
        if let Some(date_before) = filter.date_before.clone() {
            qry.push(" AND date <= ").push_bind(date_before);
        }
        if let Some(date_after) = filter.date_after.clone() {
            qry.push(" AND date >= ").push_bind(date_after);
        }

        let transactions: Vec<Transaction> = qry.build_query_as().fetch_all(&mut *conn).await?;
        Ok(transactions)
    }

    /// Fetch a single transaction by ID
    pub async fn get(db: &Connection, id: u32) -> Result<Transaction> {
        let filter = TransactionFilter {
            member_id: Some(id),
            ..TransactionFilter::default()
        };
        let transaction: Transaction = Self::filter(db, &filter)
            .await?
            .pop()
            .ok_or_else(|| Error::NotFound)?;
        Ok(transaction)
    }

    /// Create transaction
    pub async fn insert(&self, db: &Connection) -> Result<Transaction> {
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

    /// Delete a transaction
    pub async fn delete(&self, db: &Connection) -> Result<()> {
        let mut conn = db.lock().await;
        QueryBuilder::<Sqlite>::new("DELETE FROM transactions WHERE id = ")
           .push_bind(self.id)
           .build()
           .execute(&mut *conn).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{connection, members::Member};

    #[tokio::test]
    async fn test_transaction_insert() {
        let (_handle, conn) = connection::open_test().await;

        // Create test member
        let m = Member{
            name: "Testmember".to_string(),
            ..Default::default()
        };
        let m = m.insert(&conn).await.unwrap();

        let date = NaiveDate::from_ymd_opt(2021, 3, 9).unwrap();

        // Create transaction for member
        let tx = Transaction {
            member_id: m.id,
            date: date,
            account_name: "Testmember AccountName".to_string(),
            amount: 23.0,
            description: "Mitgliedsbeitrag".to_string(),
            ..Default::default()
        };
        
        let tx = tx.insert(&conn).await.unwrap();
        assert!(tx.id > 0);
        assert_eq!(tx.member_id, m.id);
        assert_eq!(tx.date, date);
        assert_eq!(tx.account_name, "Testmember AccountName");
        assert_eq!(tx.amount, 23.0);
        assert_eq!(tx.description, "Mitgliedsbeitrag");
    }

    #[tokio::test]
    async fn test_transaction_delete() {
        let (_handle, conn) = connection::open_test().await;

        // Create test member
        let m = Member{
            name: "Testmember".to_string(),
            ..Default::default()
        };
        let m = m.insert(&conn).await.unwrap();

        // Create transaction for member
        let tx = Transaction {
            member_id: m.id,
            ..Default::default()
        };
        let tx = tx.insert(&conn).await.unwrap();

        let tx_id = tx.id;

        // Delete transaction
        tx.delete(&conn).await.unwrap();

        // This should now fail
        let tx = Transaction::get(&conn, tx_id).await;
        assert!(tx.is_err());
    }

    #[tokio::test]
    async fn test_transaction_filter() {
        let (_handle, conn) = connection::open_test().await;

        // Create test members
        let m1 = Member{
            name: "Testmember1".to_string(),
            ..Default::default()
        };
        let m1 = m1.insert(&conn).await.unwrap();
        let m2 = Member{
            name: "Testmember2".to_string(),
            ..Default::default()
        };
        let m2 = m2.insert(&conn).await.unwrap();

        // Create transaction for members
        let tx = Transaction {
            member_id: m1.id,
            ..Default::default()
        };
        tx.insert(&conn).await.unwrap();
        let tx = Transaction {
            member_id: m2.id,
            ..Default::default()
        };
        tx.insert(&conn).await.unwrap();

        // Filter transactions
        let filter = TransactionFilter {
            member_id: Some(m1.id),
            ..Default::default()
        };
        let txs = Transaction::filter(&conn, &filter).await.unwrap();
        assert_eq!(txs.len(), 1);
    }

}

