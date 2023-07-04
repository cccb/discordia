use chrono::NaiveDate;
use sqlx::FromRow;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct State {
    pub accounts_calculated_at: NaiveDate,
}

