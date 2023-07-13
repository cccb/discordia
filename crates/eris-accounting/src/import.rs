use async_trait::async_trait;
use anyhow::Result;

use eris_data::{
    Update,
    Insert,
    Member,
    Retrieve,
    Transaction,
    BankImportRule,
    BankImportRuleFilter,
    Query,
};

use crate::member_fees::MemberFee;

#[async_trait]
pub trait ImportTransaction {
    /// Import transactions from a bank statement
    async fn import<DB>(&self, db: &DB) -> Result<()>
    where
        DB: Insert<Transaction> +
            Update<Member> +
            Query<BankImportRule> +
            Send + Sync;
}

/// Make default import rule for a member
fn make_default_rule(member: &Member, iban: &str) -> BankImportRule {
    BankImportRule{
        member_id: member.id,
        iban: iban.to_string(),
        ..Default::default()
    };
}

#[async_trait]
impl ImportTransaction for BankTransaction {
    async fn import<DB>(&self, db: &DB) -> Result<()>
    where
        DB: Insert<Transaction> +
            Update<Member> +
            Retrieve<Member> +
            Query<BankImportRule> +
            Send + Sync
    {
        // Check if there is are bank import rules for the iban
        let rules: Vec<BankImportRule> = db.query(BankImportRuleFilter{
            iban: Some(self.iban.clone()),
            ..Default::default()
        }).await?; 
        
        // If there are no rules, we make up a default rule
        // for a member with the same name as the account.
        /*
        let rules = if rules.is_empty() {
            let rule = db.insert(
            ).await?;
            vec![rule]
        } else {
            rules
        }
        */

        // Iterate rules and create transactions
        for rule in rules {
           let member = rule.get_member(&db).await?;

            // Make transaction and apply to member account
            let tx = Transaction{
                date: self.date,
                amount: self.amount,
                account_name: self.name.clone(),
                description: self.subject.clone(),
                ..Default::default()
            };

            let member = member.apply_transaction(&db, tx);
        }

        Ok(())
    }
}
