
use anyhow::Result;
use chrono::{Datelike, NaiveDate};
use inquire::Confirm;
use clap::{Subcommand, Args};

use eris_db::Connection;
use eris_data::{
    Update,
    Transaction,
    MemberFilter,
    Query,
    Member,
};
use eris_accounting::{
    transactions::ApplyTransaction,
    member_fees::{
        CalculateFees,
    },
    datetime::last_month,
};

use crate::commands::Transactions;


#[derive(Subcommand, Debug)]
pub enum Accounting {
    /// Calculate account balances
    #[clap(name = "calculate")]
    Calculate(CalculateAccounts),

    /// Manage transactions
    #[clap(subcommand)]
    Transactions(Transactions),
}

impl Accounting {
    pub async fn run(self, db: &Connection) -> Result<()> {
        match self {
            Accounting::Calculate(cmd) => cmd.run(db).await,
            Accounting::Transactions(cmd) => cmd.run(db).await,
        }
    }
}

#[derive(Args, Debug)]
pub struct CalculateAccounts {
    #[clap(short, long)]
    pub id: Option<u32>,
    #[clap(short, long, default_value_t=last_month())]
    pub until: NaiveDate,
}

impl CalculateAccounts {
    /// Run the account calculations
    pub async fn run(self, db: &Connection) -> Result<()> {
        // Get current state
        let end = self.until.with_day(1).unwrap();

        // Confirm calculation
        let ok = Confirm::new(&format!(
                "Calculate account balances until {}?",
                end.format("%Y-%m")))
            .prompt()?;
        if !ok {
            return Ok(());
        }

        // Calculate fees for each members
        let members: Vec<Member> = db.query(
            &MemberFilter::default()).await?;
        for mut member in members {
            let fees = member.calculate_fees(end);
            if fees.is_empty() {
                continue; // nothing to do here.
            }

            let transactions: Vec<Transaction> = fees.into_iter()
                .map(|fee| fee.into())
                .collect();
            let num = transactions.len();
            let total = transactions.iter()
                .map(|t| t.amount)
                .sum::<f64>();


            let start = std::cmp::max(member.account_calculated_at, member.membership_start);
            let start = start.with_day(1).unwrap().format("%Y-%m");
            println!("{}: fees since {} for {} month: {}€",  member.name, start, num, total);

            // Apply transactions
            for tx in transactions {
                member = member.apply_transaction(db, tx).await?;
            }
            // Update state
            member.account_calculated_at = end;
            member = db.update(member).await?;

            println!("Current balance: {}€", member.account);
            println!();
        }

        Ok(())
    }
}
