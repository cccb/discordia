use chrono::NaiveDate;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct State {
    pub accounts_calculated_at: NaiveDate,
}

