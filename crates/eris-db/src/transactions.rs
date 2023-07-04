use anyhow::Result;
use async_trait::async_trait;
use sqlx::{QueryBuilder, Sqlite};

use eris_domain::{Delete, Insert, Query, Retrieve, Transaction, TransactionFilter};

use crate::{
    results::{Id, QueryError},
    Connection,
};

#[async_trait]
impl Query<Transaction> for Connection {
    type Filter = TransactionFilter;

    async fn query(&self, filter: &Self::Filter) -> Result<Vec<Transaction>> {
        let mut conn = self.lock().await;
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
        if let Some(id) = filter.id {
            qry.push(" AND id = ").push_bind(id);
        }
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
}

#[async_trait]
impl Retrieve<Transaction> for Connection {
    type Filter = TransactionFilter;
    async fn retrieve(&self, filter: &Self::Filter) -> Result<Transaction> {
        let transaction: Transaction = self
            .query(filter)
            .await?
            .pop()
            .ok_or_else(|| QueryError::NotFound)?;
        Ok(transaction)
    }
}

#[async_trait]
impl Insert<Transaction> for Connection {
    async fn insert(&self, transaction: Transaction) -> Result<Transaction> {
        let insert: Id<u32> = {
            let mut conn = self.lock().await;
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
                .push_bind(transaction.member_id)
                .push_bind(transaction.date)
                .push_bind(&transaction.account_name)
                .push_bind(transaction.amount)
                .push_bind(&transaction.description);

            qry.push(") RETURNING id ")
                .build_query_as()
                .fetch_one(&mut *conn)
                .await?
        };
        let filter = TransactionFilter {
            id: Some(insert.id),
            ..Default::default()
        };
        self.retrieve(&filter).await
    }
}

#[async_trait]
impl Delete<Transaction> for Connection {
    /// Delete a transaction
    async fn delete(&self, tx: Transaction) -> Result<()> {
        let mut conn = self.lock().await;
        QueryBuilder::<Sqlite>::new("DELETE FROM transactions WHERE id = ")
           .push_bind(tx.id)
           .build()
           .execute(&mut *conn).await?;
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::NaiveDate;

    use eris_domain::Member;

    #[tokio::test]
    async fn test_transaction_insert() {
        let db = Connection::open_test().await;

        // Create test member
        let m = Member {
            name: "Testmember".to_string(),
            ..Default::default()
        };
        // let m = m.insert(&conn).await.unwrap();
        let m = db.insert(m).await.unwrap();

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

        let tx = db.insert(tx).await.unwrap();
        assert!(tx.id > 0);
        assert_eq!(tx.member_id, m.id);
        assert_eq!(tx.date, date);
        assert_eq!(tx.account_name, "Testmember AccountName");
        assert_eq!(tx.amount, 23.0);
        assert_eq!(tx.description, "Mitgliedsbeitrag");
    }

    #[tokio::test]
    async fn test_transaction_delete() {
        let db = Connection::open_test().await;

        // Create test member
        let m = Member{
            name: "Testmember".to_string(),
            ..Default::default()
        };
        let m = db.insert(m).await.unwrap();

        // Create transaction for member
        let tx = Transaction {
            member_id: m.id,
            ..Default::default()
        };
        let tx = db.insert(tx).await.unwrap();
        let tx_id = tx.id;

        // Delete transaction
        db.delete(tx).await.unwrap();

        // This should now fail
        let tx: Result<Transaction> = db.retrieve(&TransactionFilter{
            id: Some(tx_id),
            ..Default::default()
        }).await;
        assert!(tx.is_err());
    }

    #[tokio::test]
    async fn test_transaction_filter() {
        let db = Connection::open_test().await;

        // Create test members
        let m1 = Member{
            name: "Testmember1".to_string(),
            ..Default::default()
        };
        let m1 = db.insert(m1).await.unwrap();
        let m2 = Member{
            name: "Testmember2".to_string(),
            ..Default::default()
        };
        let m2 = db.insert(m2).await.unwrap();

        // Create transaction for members
        let tx = Transaction {
            member_id: m1.id,
            ..Default::default()
        };
        db.insert(tx).await.unwrap();

        let tx = Transaction {
            member_id: m2.id,
            ..Default::default()
        };
        db.insert(tx).await.unwrap();

        // Filter transactions
        let filter = TransactionFilter {
            member_id: Some(m1.id),
            ..Default::default()
        };
        let txs: Vec<Transaction> = db.query(&filter).await.unwrap();
        assert_eq!(txs.len(), 1);
    }

}
