use anyhow::{anyhow, Result};
use chrono::NaiveDate;
use clap::{Args, Subcommand};

use eris_data::{Member, MemberFilter, Query, Transaction, TransactionFilter,  Retrieve};
use eris_db::Connection;

#[derive(Subcommand, Debug)]
pub enum Transactions {
    /// List transactions
    List(ListTransactions),
}

impl Transactions {
    pub async fn run(self, conn: &Connection) -> Result<()> {
        match self {
            Transactions::List(cmd) => cmd.run(conn).await,
        }
    }
}

#[derive(Args, Debug)]
pub struct ListTransactions {
    #[clap(long)]
    pub member_id: Option<u32>,
    #[clap(long)]
    pub member_name: Option<String>,
    #[clap(short, long)]
    pub after_date: Option<NaiveDate>,
    #[clap(short, long)]
    pub before_date: Option<NaiveDate>,
}

impl ListTransactions {
    pub async fn run(self, db: &Connection) -> Result<()> {
        // Build filter
        let mut filter = TransactionFilter::default();

        if let Some(id) = self.member_id {
            filter.member_id = Some(id);
        }
        if let Some(name) = self.member_name {
            let members: Vec<Member> = db.query(&MemberFilter{
                name: Some(name),
                ..Default::default()
            }).await?;
            let member = members.first().ok_or(anyhow!("member not found"))?;
            filter.member_id = Some(member.id);
        }

        if let Some(date) = self.after_date {
            filter.date_after = Some(date);
        }
        if let Some(date) = self.before_date {
            filter.date_before = Some(date);
        }

        // Query and print transctions
        let transactions: Vec<Transaction> = db.query(&filter).await?;
        println!(
            "{:>4}\t{:<15}\t{:<30}\t{:<40}\t{:<12}\t{}",
            "ID", "Date", "Member", "Account", "Amount", "Description"
        );
        println!("{:-<180}", "-");
        for tx in transactions {
            let member: Member = db.retrieve(tx.member_id).await?;
            println!(
                "{:>4}\t{:<15}\t{:<30}\t{:<40}\t{:<12.2}\t{}",
                tx.id, tx.date, member.name, tx.account_name, tx.amount, tx.description
            );
        }

        Ok(())
    }
}
