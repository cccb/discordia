use async_trait::async_trait;
use chrono::NaiveDate;
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
    TransactionFilter,
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

    #[error("insufficient amount for split transaction")]
    InsufficientAmountForSplit(BankTransaction),

    #[error("import date {0} is older than last inbound transaction date {1}")]
    NewerTransactionsPresent(NaiveDate, NaiveDate),

    #[error(transparent)]
    Error(#[from] anyhow::Error),
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
pub trait ImportTransaction {
    /// Import transactions from a bank statement
    async fn import<DB>(self, db: &DB) -> Result<(), BankImportError>
    where
        DB: Insert<Transaction> +
            Insert<BankImportRule> +
            Update<Member> +
            Retrieve<Member, Key=u32> +
            Query<BankImportRule, Filter=BankImportRuleFilter> +
            Query<Member, Filter=MemberFilter> +
            Send + Sync;
}


#[async_trait]
impl ImportTransaction for BankTransaction {
    async fn import<DB>(self, db: &DB) -> Result<(), BankImportError>
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
            vec![make_default_rule(db, &self).await?]
        } else {
            rules
        };

        // Total amount of the transaction, which will be split
        // in case there is a split rule. The left-over will be
        // applied to the first rule.
        let mut total_amount = self.amount;
        let mut transactions: Vec<(Member, Transaction)> = vec![];

        // Iterate rules and create transactions
        for rule in &rules {
            // Check if the rule matches the subject
            if Some(false) == rule.match_subject(&self.subject) {
                println!(
                    "excluding transaction {} for {} because \
                    subject rule does not match {}",
                    self.subject,
                    self.name,
                    rule.match_subject.clone().unwrap());
                continue;
            }

            // In case we have a split transaction, we have to deduce
            // the amount from the total amount
            let mut amount = total_amount;
            let mut subject = self.subject.clone();
            if let Some(split_amount) = rule.split_amount {
                if split_amount > total_amount {
                    return Err(BankImportError::InsufficientAmountForSplit(
                        self.clone()));
                }
                amount = split_amount;
                subject += " (split)";
            }

            // Make transaction and apply to member account
            let tx = Transaction{
                date: self.date,
                amount: amount,
                account_name: self.name.clone(),
                description: self.subject.clone(),
                ..Default::default()
            };
            let member = rule.get_member(db).await?;
            transactions.push((member, tx));
            total_amount -= amount;
        }
    
        // Apply transactions to member accounts
        for (member, tx) in transactions {
            member.apply_transaction(db, tx).await?;
        }
    
        if total_amount <= 0.0 {
            return Ok(()); // we are done here.
        }

        // We have a left-over amount, which we will
        // apply to the first rule.
        if let Some(rule) = rules.first() {
            let subject = format!("{} (overflow)", self.subject);
            let member = rule.get_member(db).await?;
            let tx = Transaction{
                date: self.date,
                amount: total_amount,
                account_name: self.name.clone(),
                description: subject, 
                ..Default::default()
            };
            member.apply_transaction(db, tx).await?;
        }

        Ok(())
    }
}

/// Plausibility check: Test before importing a bank
/// CSV file, that there are no transactions with a
/// newer date in the database.
pub async fn check_import_date<DB>(
    db: &DB,
    date: NaiveDate,
) -> Result<(), BankImportError> 
where
    DB: Query<Transaction, Filter=TransactionFilter> +
        Send + Sync
{
    let last_date = db.query(&TransactionFilter{
        date_after: Some(date),
        ..Default::default()
    }).await?.iter().map(|tx| tx.date).max();
    if let Some(last_date) = last_date {
        if last_date >= date {
            return Err(BankImportError::NewerTransactionsPresent(
                date, last_date,
            ));
        }
    }
    Ok(())
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

    #[tokio::test]
    async fn test_check_import_date() {
        let db = Connection::open_test().await;
        // Insert a testmember and a transaction
        let member = db.insert(Member{
            name: "Test Member".to_string(),
            ..Default::default()
        }).await.unwrap();

        member.apply_transaction(&db, Transaction{
            date: NaiveDate::from_ymd_opt(2023, 5, 10).unwrap(),
            amount: 23.0,
            ..Default::default()
        }).await.unwrap();

        // Should be ok:
        check_import_date(
            &db, NaiveDate::from_ymd_opt(2023, 6, 1).unwrap()
        ).await.unwrap();

        // Previous date should fail:
        let result = check_import_date(
            &db, NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()
        ).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_import_bank_transaction() {
        let db = Connection::open_test().await;
        // Insert a testmember and a transaction
        let member = db.insert(Member{
            name: "Test Member".to_string(),
            ..Default::default()
        }).await.unwrap();

        // Bank transaction for test member
        let tx = BankTransaction{
            id: 1,
            name: "Test Member".to_string(),
            iban: "DE1111111111111".to_string(),
            amount: 23.0,
            date: NaiveDate::from_ymd_opt(2023, 5, 10).unwrap(),
            subject: "Test Transaction".to_string(),
        };

        // Import the transaction
        tx.import(&db).await.unwrap();

        let member: Member = db.retrieve(member.id).await.unwrap();
        assert_eq!(member.account, 23.0);
    }

}

