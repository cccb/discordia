use async_trait::async_trait;
use anyhow::Result;
use thiserror::Error as ThisError;

use eris_banking::BankTransaction;
use eris_data::{
    Update,
    Insert,
    Member,
    Retrieve,
    MemberFilter,
    Transaction,
    BankImportRule,
    BankImportRuleFilter,
    Query,
};

use crate::transactions::ApplyTransaction;

/// BankImportError type
#[derive(ThisError, Debug)]
pub enum BankImportError {
    #[error("could not resolve member for iban")]
    AccountMatchFailed(BankTransaction),

    #[error(transparent)]
    Error(#[from] anyhow::Error),
}


#[async_trait]
pub trait ImportTransaction {
    /// Import transactions from a bank statement
    async fn import<DB>(&self, db: &DB) -> Result<(), BankImportError>
    where
        DB: Insert<Transaction> +
            Insert<BankImportRule> +
            Update<Member> +
            Retrieve<Member, Key=u32> +
            Query<BankImportRule, Filter=BankImportRuleFilter> +
            Query<Member, Filter=MemberFilter> +
            Send + Sync;
}

/// Lookup member by account name and create a default rule
async fn make_default_rule<DB>(
    db: &DB,
    tx: &BankTransaction,
) -> Result<BankImportRule, BankImportError> 
where
    DB: Insert<BankImportRule> +
        Query<Member, Filter=MemberFilter> +
        Send + Sync
{
    let members = db.query(&MemberFilter{
        name: Some(tx.name.clone()),
        ..Default::default()
    }).await?;
    let member = if members.len() == 1 {
        Ok(members[0].clone())
    } else {
        Err(BankImportError::AccountMatchFailed(tx.clone()))
    }?;

    if members.len() != 1 {
        return Err(BankImportError::AccountMatchFailed(tx.clone()));
    }

    // Create bank import rule
    let rule = db.insert(BankImportRule{
        member_id: member.id,
        iban: tx.iban.clone(),
        ..Default::default()
    }).await?;

    Ok(rule)
}


#[async_trait]
impl ImportTransaction for BankTransaction {
    async fn import<DB>(&self, db: &DB) -> Result<(), BankImportError>
    where
        DB: Insert<Transaction> +
            Insert<BankImportRule> +
            Update<Member> +
            Retrieve<Member, Key=u32> +
            Query<BankImportRule, Filter=BankImportRuleFilter> +
            Query<Member, Filter=MemberFilter> +
            Send + Sync
    {
        // Check if there is are bank import rules for the iban
        let rules: Vec<BankImportRule> = db.query(&BankImportRuleFilter{
            iban: Some(self.iban.clone()),
            ..Default::default()
        }).await?; 
        
        // If there are no rules, we make up a default rule
        // for a member with the same name as the account.
        let rules = if rules.is_empty() {
            vec![make_default_rule(db, self).await?]
        } else {
            rules
        };

        // Iterate rules and create transactions
        for rule in rules {
           let member = rule.get_member(db).await?;

            // Make transaction and apply to member account
            let tx = Transaction{
                date: self.date,
                amount: self.amount,
                account_name: self.name.clone(),
                description: self.subject.clone(),
                ..Default::default()
            };

            member.apply_transaction(db, tx).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eris_db::Connection;

    #[tokio::test]
    async fn test_make_default_rule_member_match() {
        let db = Connection::open_test().await;
        // Insert test member and try to derive a rule
        let member = db.insert(Member{
            name: "Test Member".to_string(),
            ..Default::default()
        }).await.unwrap();
        let tx = BankTransaction{
            name: "test member".to_string(),
            iban: "DE1231231111111111".to_string(),
            ..Default::default()
        };
        // This should work because we have a matching member
        let rule = make_default_rule(&db, &tx).await.unwrap();
        assert_eq!(rule.member_id, member.id);
        assert_eq!(rule.iban, tx.iban);
    }

    #[tokio::test]
    async fn test_make_default_rule_member_no_match() {
        let db = Connection::open_test().await;
        // Insert test member and try to derive a rule
        db.insert(Member{
            name: "Test Member".to_string(),
            ..Default::default()
        }).await.unwrap();
        let tx = BankTransaction{
            name: "best member".to_string(),
            iban: "DE1231231111111111".to_string(),
            ..Default::default()
        };
        // This should work because we have a matching member
        let rule = make_default_rule(&db, &tx).await;
        assert!(rule.is_err());
        match rule {
            Err(BankImportError::AccountMatchFailed(tx)) => {
                assert_eq!(tx.name, "best member");
            },
            _ => panic!("unexpected error"),
        }
    }

}

