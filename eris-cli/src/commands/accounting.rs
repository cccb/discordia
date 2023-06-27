
use anyhow::Result;
use chrono::NaiveDate;
use clap::Args;

use eris_db::Connection;
use eris_accounting::datetime::last_month;


#[derive(Args, Debug)]
pub struct CalculateAccounts {
    #[clap(short, long)]
    pub id: Option<u32>,
    #[clap(short, long, default_value_t=last_month())]
    pub until: NaiveDate,
}

impl CalculateAccounts {
    /// Run the account calculations
    pub async fn run(&self, conn: &Connection) -> Result<()> {
        Ok(())
    }
}
