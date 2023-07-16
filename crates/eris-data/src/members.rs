use anyhow::Result;
use chrono::NaiveDate;
use sqlx::FromRow;
use serde::{Serialize, Deserialize};

use crate::{
    BankImportRuleFilter,
    BankImportRule,
    Query,
    Transaction,
    TransactionFilter,
};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MemberFilter {
    pub id: Option<u32>,
    pub name: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Default, FromRow, Serialize, Deserialize)]
pub struct Member {
    pub id: u32,
    pub name: String,
    pub email: String,
    pub notes: String,
    pub membership_start: NaiveDate,
    pub membership_end: Option<NaiveDate>,
    pub fee: f64,
    pub interval: u8,
    pub last_payment: NaiveDate,
    pub account: f64,
}

impl Member {

    /// Get related bank import rules for a member
    pub async fn get_bank_import_rules<DB>(
        &self,
        db: &DB,
    ) -> Result<Vec<BankImportRule>>
    where
         DB: Query<BankImportRule, Filter=BankImportRuleFilter>,
    {
        let rules = db.query(&BankImportRuleFilter{
            member_id: Some(self.id),
            ..Default::default()
        }).await?;
        Ok(rules)
    }

    pub async fn get_transactions<DB>(
        &self,
        db: &DB,
    ) -> Result<Vec<Transaction>>
    where
         DB: Query<Transaction, Filter=TransactionFilter>,
    {
        let transactions = db.query(&TransactionFilter{
            member_id: Some(self.id),
            ..Default::default()
        }).await?;
        Ok(transactions)
    }

    // Check if member is active
    pub fn is_active(&self, date: NaiveDate) -> bool {
        if date < self.membership_start {
            return false;
        }
        if let Some(end) = self.membership_end {
            if date > end {
                return false;
            }
        }
        true
    }
}

