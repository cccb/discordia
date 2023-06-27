
use chrono::NaiveDate;
use clap::Args;

#[derive(Args, Debug)]
pub struct AddMember{
    #[clap(short, long)]
    pub name: String,
    #[clap(short, long)]
    pub email: String,
    #[clap(short='c', long)]
    pub notes: Option<String>,
    #[clap(long)]
    pub membership_start: Option<NaiveDate>,
    #[clap(short, long)]
    pub fee: Option<f64>,
    #[clap(short='p', long, default_value_t=1)]
    pub interval: u8,
    #[clap(short, long, default_value_t=0.0)]
    pub account: f64,
}


#[derive(Args, Debug)]
pub struct UpdateMember{
    #[clap(short, long)]
    pub id: u32,
    #[clap(short, long)]
    pub name: Option<String>,
    #[clap(short, long)]
    pub email: Option<String>,
    #[clap(short='c', long)]
    pub notes: Option<String>,
    #[clap(long)]
    pub membership_start: Option<NaiveDate>,
    #[clap(long)]
    pub membership_end: Option<NaiveDate>,
    #[clap(short, long)]
    pub fee: Option<f64>,
    #[clap(short='p', long)]
    pub interval: Option<u8>,
    #[clap(short, long)]
    pub account: Option<f64>,
}

#[derive(Args, Debug)]
pub struct DeleteMember{
    #[clap(short, long)]
    pub id: u32,
}
