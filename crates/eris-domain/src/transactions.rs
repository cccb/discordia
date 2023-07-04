use chrono::NaiveDate;
use sqlx::FromRow;
use serde::{Serialize, Deserialize};


#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TransactionFilter {
    pub member_id: Option<u32>,
    pub date: Option<NaiveDate>,
    pub date_before: Option<NaiveDate>,
    pub date_after: Option<NaiveDate>,
}

#[derive(Debug, Default, Clone, FromRow, Serialize, Deserialize)]
pub struct Transaction {
    pub id: u32,
    pub member_id: u32,
    pub date: NaiveDate,
    pub account_name: String,
    pub amount: f64,
    pub description: String,
}

