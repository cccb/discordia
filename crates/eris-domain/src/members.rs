use chrono::NaiveDate;
use sqlx::FromRow;
use serde::{Serialize, Deserialize};

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

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MemberFilter {
    pub id: Option<u32>,
    pub name: Option<String>,
    pub email: Option<String>,
}
