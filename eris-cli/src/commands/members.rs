
use anyhow::Result;
use chrono::NaiveDate;
use clap::Args;

use eris_db::{Member, MemberFilter, Connection};

#[derive(Args, Debug)]
pub struct ListMembers {
    #[clap(short, long)]
    pub id: Option<u32>,
    #[clap(short, long)]
    pub name: Option<String>,
    #[clap(short, long)]
    pub email: Option<String>,
}

impl ListMembers {
    /// Run the command and list members
    pub async fn run(self, conn: &Connection) -> Result<()> {
        // Create member filter
        let filter = MemberFilter{
            id: self.id,
            name: self.name,
            email: self.email,
            ..Default::default()
        };

        let members = Member::filter(conn, &filter).await?;
        for member in members {
            println!("{:?}", member);
        }

        Ok(())
    }
}

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

impl AddMember {
    /// Run the command and add a member to the database
    pub async fn run(&self, conn: &Connection) -> Result<()> {
        Ok(())
    }
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

impl UpdateMember {
    /// Run command and update a member
    pub async fn run(&self, conn: &Connection) -> Result<()> {
        Ok(())
    }
}


#[derive(Args, Debug)]
pub struct DeleteMember{
    #[clap(short, long)]
    pub id: u32,
}


impl DeleteMember {
    pub async fn run(&self, conn: &Connection) -> Result<()> {
        Ok(())
    }
}
