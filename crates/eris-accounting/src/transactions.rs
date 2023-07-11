use async_trait::async_trait;
use anyhow::Result;

use eris_data::{
    Retrieve,
    Update,
    Insert,
    Member,
    Transaction,
};

#[async_trait]
pub trait ApplyTransaction {
    /// Apply a transaction for a member
    async fn apply_transaction<DB>(self, db: &DB, tx: Transaction) -> Result<Member>
    where
        DB: Insert<Transaction> +
            Retrieve<Member, Key=u32> +
            Update<Member> +
            Copy + Send + Sync;
}


#[async_trait]
impl ApplyTransaction for Member {
    async fn apply_transaction<DB>(self, db: &DB, tx: Transaction) -> Result<Member>
    where
        DB: Insert<Transaction> +
            Retrieve<Member, Key=u32> +
            Update<Member> +
            Copy + Send + Sync
    {
        let mut member = self.clone();
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
