use anyhow::Result;
use chrono::NaiveDate;
use thiserror::Error as ThisError;

use eris_db::Connection;
use eris_data::{
    Query,
    Insert,
    Update,
    Transaction,
    BankImportRule,
    BankImportRuleFilter,
    Member,
    MemberFilter,
};
use eris_accounting::transactions::ApplyTransaction;

#[derive(Debug, Default, Clone)]
pub struct BankTransaction {
    pub num: u32,
    pub date: NaiveDate,
    pub name: String,
    pub iban: String,
    pub amount: f64,
    pub subject: String,
}


/// BankImportError type
#[derive(ThisError, Debug)]
pub enum BankImportError {
    #[error("could not resolve member for iban")]
    AccountMatchFailed(BankTransaction),

    #[error("insufficient amount for split transaction")]
    InsufficientAmountForSplit(BankTransaction),

    #[error("a more recent transaction ({0}) is present in database")]
    MoreRecentTransactionPresent(String),

    #[error(transparent)]
    Error(#[from] anyhow::Error),
}

impl BankTransaction {
    /// Check if the member has a more recent
    /// transaction.
    fn check_last_member_transcation(
        &self,
        member: &Member,
    ) -> Result<(), BankImportError> {
        let has_recent_date = member.last_bank_transaction_at >= self.date;
        let has_newer_serial = member.last_bank_transaction_number >= self.num;
        if has_recent_date && has_newer_serial {
            return Err(BankImportError::MoreRecentTransactionPresent(
                format!("{}-{} >= {}-{}",
                    member.last_bank_transaction_at,
                    member.last_bank_transaction_number,
                    self.date,
                    self.num)));
        }
        Ok(())
    }
    
    /// Lookup member by account name and create a default rule
    async fn make_default_rule(
        &self,
        db: &Connection,
    ) -> Result<BankImportRule, BankImportError> {
        let members: Vec<Member> = db.query(&MemberFilter{
            name: Some(self.name.clone()),
            ..Default::default()
        }).await?;
        let member = if members.len() == 1 {
            Ok(members[0].clone())
        } else {
            Err(BankImportError::AccountMatchFailed(self.clone()))
        }?;

        if members.len() != 1 {
            return Err(BankImportError::AccountMatchFailed(self.clone()));
        }

        // Create bank import rule
        let rule = db.insert(BankImportRule{
            member_id: member.id,
            iban: self.iban.clone(),
            ..Default::default()
        }).await?;

        Ok(rule)
    }

    /// Import bank transaction into database
    pub async fn import(self, db: &Connection) -> Result<(), BankImportError>
    {
        // Check if there is are bank import rules for the iban
        let rules: Vec<BankImportRule> = db.query(&BankImportRuleFilter{
            iban: Some(self.iban.clone()),
            ..Default::default()
        }).await?; 
        
        // If there are no rules, we make up a default rule
        // for a member with the same name as the account.
        let rules = if rules.is_empty() {
            vec![self.make_default_rule(db).await?]
        } else {
            rules
        };

        // Total amount of the transaction, which will be split
        // in case there is a split rule. The left-over will be
        // applied to the first rule.
        let mut total_amount = self.amount;
        let mut transactions: Vec<(Member, Transaction, u32)> = vec![];

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

            // Check if there is a more recent transaction
            let member = rule.get_member(db).await?;
            self.check_last_member_transcation(&member)?;

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

            // Make transaction and queue application
            let tx = Transaction{
                date: self.date,
                amount: amount,
                account_name: self.name.clone(),
                description: self.subject.clone(),
                ..Default::default()
            };
            transactions.push((member, tx, self.num));
            total_amount -= amount;
        }
    
        // Apply transactions to member accounts
        for (member, tx, num) in transactions {
            let mut member = member.apply_transaction(
                db, tx.clone()).await?;
            member.last_bank_transaction_at = tx.date;
            member.last_bank_transaction_number = num;
            db.update(member).await?;
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


#[cfg(test)]
mod tests {
    use super::*;
    use eris_data::{TransactionFilter, Retrieve};
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
        let rule = tx.make_default_rule(&db).await.unwrap();
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
        let rule = tx.make_default_rule(&db).await;
        assert!(rule.is_err());
        match rule {
            Err(BankImportError::AccountMatchFailed(tx)) => {
                assert_eq!(tx.name, "best member");
            },
            _ => panic!("unexpected error"),
        }
    }

    #[tokio::test]
    async fn test_check_last_member_tx() {
        let db = Connection::open_test().await;
        let member = db.insert(Member{
            name: "Test Member".to_string(),
            last_bank_transaction_at: NaiveDate::from_ymd_opt(2023, 2, 3).unwrap(),
            last_bank_transaction_number: 23,
            ..Default::default()
        }).await.unwrap();

        // Transaction with a date before the last transaction
        let tx = BankTransaction{
            date: NaiveDate::from_ymd_opt(2023, 2, 2).unwrap(),
            num: 1,
            ..Default::default()
        };
        let res = tx.check_last_member_transcation(&member);
        assert!(res.is_err());

        // Transaction on the same day, but with a lower number
        let tx = BankTransaction{
            date: NaiveDate::from_ymd_opt(2023, 2, 3).unwrap(),
            num: 22,
            ..Default::default()
        };
        let res = tx.check_last_member_transcation(&member);
        assert!(res.is_err());

        // Transaction on the same day, but with a higher number
        let tx = BankTransaction{
            date: NaiveDate::from_ymd_opt(2023, 2, 3).unwrap(),
            num: 24,
            ..Default::default()
        };
        let res = tx.check_last_member_transcation(&member);
        assert!(res.is_ok());

        // Transaction on the next day, but with lower number
        // as last transaction
        let tx = BankTransaction{
            date: NaiveDate::from_ymd_opt(2023, 2, 4).unwrap(),
            num: 2,
            ..Default::default()
        };
        let res = tx.check_last_member_transcation(&member);
        assert!(res.is_ok());

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
            num: 42,
            name: "Test Member".to_string(),
            iban: "DE1111111111111".to_string(),
            amount: 23.0,
            date: NaiveDate::from_ymd_opt(2023, 5, 10).unwrap(),
            subject: "Test Transaction".to_string(),
        };

        // Import the transaction
        tx.clone().import(&db).await.unwrap();

        let member: Member = db.retrieve(member.id).await.unwrap();
        assert_eq!(member.account, 23.0);
        assert_eq!(member.last_bank_transaction_at, tx.date);
        assert_eq!(member.last_bank_transaction_number, tx.num);
    }

    #[tokio::test]
    async fn test_import_bank_transaction_split_iban() {
        let db = Connection::open_test().await;
        let m1 = db.insert(Member{
            name: "Test Member".to_string(),
            ..Default::default()
        }).await.unwrap();
        let m2 = db.insert(Member{
            name: "Best Member".to_string(),
            ..Default::default()
        }).await.unwrap();

        // They share an account and there is a bank import
        // rule for this
        db.insert(BankImportRule{
            member_id: m1.id,
            iban: "DE2342".to_string(),
            split_amount: Some(10.0),
            ..Default::default()
        }).await.unwrap();
        db.insert(BankImportRule{
            member_id: m2.id,
            iban: "DE2342".to_string(),
            split_amount: Some(20.0),
            ..Default::default()
        }).await.unwrap();

        // Bank transaction for test member
        let tx = BankTransaction{
            num: 1,
            name: "Dr. M. Ber, B. Member".to_string(),
            iban: "DE2342".to_string(),
            amount: 32.0,
            date: NaiveDate::from_ymd_opt(2023, 5, 10).unwrap(),
            subject: "Mitgliedsbeitrag fuer beide".to_string(),
        };

        // Import the transaction
        tx.import(&db).await.unwrap();

        // There should now be three transactions:
        let tx: Vec<Transaction> = db.query(&TransactionFilter{
            ..Default::default()
        }).await.unwrap();
        assert_eq!(tx.len(), 3);

        // M1 balance should be 10 + 2 overflow
        let m1: Member = db.retrieve(m1.id).await.unwrap();
        assert_eq!(m1.account, 12.0);

        // M2 balance should be 20
        let m2: Member = db.retrieve(m2.id).await.unwrap();
        assert_eq!(m2.account, 20.0);
    }
}
