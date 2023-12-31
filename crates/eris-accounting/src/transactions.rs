use async_trait::async_trait;
use anyhow::Result;

use eris_db::Connection;
use eris_data::{
    Update,
    Insert,
    Member,
    Transaction,
};

use crate::member_fees::MemberFee;

impl From<MemberFee> for Transaction {
    /// Convert a member fee into a transaction.
    fn from(fee: MemberFee) -> Self {
        Transaction{
            amount: -fee.amount,
            date: fee.date,
            account_name: "memberhip fee".to_string(),
            description: fee.describe(),
            ..Default::default()
        }
    }
}

#[async_trait]
pub trait ApplyTransaction {
    async fn apply_transaction(
        self,
        db: &Connection,
        tx: Transaction,
    ) -> Result<Member>;
}

#[async_trait]
impl ApplyTransaction for Member {
    /// Apply a transaction and update the member's
    /// account balance.
    async fn apply_transaction(
        self,
        db: &Connection,
        tx: Transaction,
    ) -> Result<Member> {
        let mut member = self; 
        let tx = Transaction{
            member_id: member.id,
            ..tx
        };
        let tx = db.insert(tx).await?;

        member.account += tx.amount;
        let member = db.update(member).await?;

        Ok(member)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[tokio::test]
    async fn test_apply_transaction() {
        let db = Connection::open_test().await;
        let member = db.insert(Member{
            account: 100.0,
            name: "test".to_string(),
            ..Default::default()
        }).await.unwrap();

        let tx = Transaction{
            amount: -23.42,
            account_name: "memberhip fee".to_string(),
            description: "monthly membership fee for ...".to_string(),
            ..Default::default()
        };

        let member = member.apply_transaction(&db, tx).await.unwrap();
        assert_eq!(member.account, 76.58);

        // Get member transactions
        let txs = member.get_transactions(&db).await.unwrap();
        assert_eq!(txs.len(), 1);
        println!("txs: {:?}", txs);
    }

    #[tokio::test]
    async fn test_tx_from_fee() {
        let db = Connection::open_test().await;
        let member = db.insert(Member{
            account: 100.0,
            name: "test".to_string(),
            ..Default::default()
        }).await.unwrap();

        let tx: Transaction = MemberFee{
            amount: 23.42,
            date: NaiveDate::from_ymd_opt(2020, 5, 23).unwrap(),
        }.into();

        let member = member.apply_transaction(&db, tx).await.unwrap();
        assert_eq!(member.account, 76.58);

        // Get member transactions
        let txs = member.get_transactions(&db).await.unwrap();
        assert_eq!(txs.len(), 1);
        println!("txs: {:?}", txs);
    }
}
