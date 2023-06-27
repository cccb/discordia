
use chrono::{NaiveDate, Local, Duration};
use clap::Args;

use eris_db::accounting::datetime::last_month;


#[derive(Args, Debug)]
pub struct CalculateAccounts {
    #[clap(short, long)]
    pub id: Option<u32>,
    #[clap(short, long, default_value_t=last_month())]
    pub until: NaiveDate,
}
