use anyhow::{anyhow, Result};
use chrono::{Datelike, NaiveDate};
use clap::{Args, Subcommand};
use inquire::Confirm;

use eris_data::{
    Member, MemberFilter, Query, Retrieve, Transaction, TransactionFilter,
};
use eris_db::Connection;

#[derive(Subcommand, Debug)]
pub enum Bank {
    /// Import a bank CSV export
    Import(BankImport),

    /// IBAN rules
    #[clap(subcommand)]
    Iban(Iban)
}

impl Bank {
    pub async fn run(self, conn: &Connection) -> Result<()> {
        match self {
            Bank::Import(import) => import.run(conn).await,
            Bank::Iban(iban) => iban.run(conn).await,
        }
    }
}

#[derive(Args, Debug)]
pub struct BankImport {
    #[clap(short, long)]
    pub file: String,

    #[clap(short, long, default_value_t = false)]
    pub dry: bool,
}

impl BankImport {
    pub async fn run(self, db: &Connection) -> Result<()> {
        Ok(())
    }
}

#[derive(Subcommand, Debug)]
pub enum Iban {
    /// List rules 
    List(IbanList),

    /// Add a rule
    #[clap(name = "add")]
    Add(IbanAdd),

    /// Remove a rule
    #[clap(name = "delete")]
    Delete(IbanRemove),
}

impl Iban {
    pub async fn run(self, conn: &Connection) -> Result<()> {
        match self {
            Iban::List(list) => list.run(conn).await,
            Iban::Add(add) => add.run(conn).await,
            Iban::Delete(delete) => delete.run(conn).await,
        }
    }
}

#[derive(Args, Debug)]
pub struct IbanList {
    #[clap(short, long)]
    pub member_id: Option<u32>,
}

impl IbanList {
    pub async fn run(self, db: &Connection) -> Result<()> {
        Ok(())
    }
}

#[derive(Args, Debug)]
pub struct IbanAdd {
    #[clap(short, long)]
    pub member_id: u32,

    #[clap(short, long)]
    pub iban: String,

    #[clap(short, long)]
    pub split_amount: Option<f64>,

    #[clap(short, long)]
    pub match_subject: Option<String>,
}

impl IbanAdd {
    pub async fn run(self, db: &Connection) -> Result<()> {
        Ok(())
    }
}

#[derive(Args, Debug)]
pub struct IbanRemove {
    #[clap(short, long)]
    pub member_id: u32,

    #[clap(short, long)]
    pub iban: String,
}

impl IbanRemove {
    pub async fn run(self, db: &Connection) -> Result<()> {
        Ok(())
    }
}
