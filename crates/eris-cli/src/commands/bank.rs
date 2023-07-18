use std::fs::File; 

use anyhow::{anyhow, Result};
use chrono::{Datelike, NaiveDate};
use clap::{Args, Subcommand};
use inquire::Confirm;

use eris_data::{
    Query,
    Retrieve,
    Insert,
    Update,
    Delete,
    Member,
    Transaction,
    TransactionFilter,
    BankImportRule,
    BankImportRuleFilter,
};
use eris_db::Connection;
use eris_accounting::import::{
    ImportTransaction,
    BankImportError,
};
use eris_banking::{
    deuba::bank_transactions,
    BankTransaction,
};

use crate::formatting::PrintFormatted;

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
}

/// Get first and last date from transactions
fn get_first_and_last_date(
    transactions: &[BankTransaction],
) -> Result<(NaiveDate, NaiveDate)> {
    let mut first = NaiveDate::from_ymd_opt(9999, 1, 1).unwrap();
    let mut last = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
    for tx in transactions {
        let date = tx.date.clone();
        if last < date {
            last = date;
        }
        if date < first {
            first = date;
        }
    }
    Ok((first, last))
}

impl BankImport {
    pub async fn run(self, db: &Connection) -> Result<()> {
        // Open CSV file
        let mut file = File::open(&self.file)?; 
        let transactions = bank_transactions::parse(&mut file)?;

        // Get first and last date from transactions
        let (first_date, last_date) = get_first_and_last_date(&transactions)?;
        let ok = Confirm::new(&format!(
            "Import transactions from {} to {}?",
            first_date,
            last_date,
        )).prompt()?;
        if !ok {
            return Ok(());
        }

        // Run import
        let mut failed_tx: Vec<(BankTransaction, BankImportError)> = vec![];
        for tx in transactions {
            match tx.clone().import(db).await {
                Ok(()) => {
                    tx.print_formatted();
                },
                Err(e) => {
                    failed_tx.push((tx, e));
                }
            } 
        }

        if !failed_tx.is_empty() {
            println!();
            println!("Failed to import transactions:");
            for (tx, e) in failed_tx {
                println!();
                tx.print_formatted();
                println!("{}", e);
            }
        }
        
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

    /// Update a rule
    #[clap(name = "set")]
    Update(IbanUpdate),

    /// Remove a rule
    #[clap(name = "delete")]
    Delete(IbanRemove),
}

impl Iban {
    pub async fn run(self, conn: &Connection) -> Result<()> {
        match self {
            Iban::List(list) => list.run(conn).await,
            Iban::Add(add) => add.run(conn).await,
            Iban::Update(update) => update.run(conn).await,
            Iban::Delete(delete) => delete.run(conn).await,
        }
    }
}

#[derive(Args, Debug)]
pub struct IbanList {
    #[clap(short, long)]
    pub member_id: Option<u32>,

    #[clap(short, long)]
    pub iban: Option<String>,
}

impl IbanList {
    pub async fn run(self, db: &Connection) -> Result<()> {
        let rules: Vec<BankImportRule> = db.query(&BankImportRuleFilter{
            member_id: self.member_id,
            iban: self.iban,
            ..Default::default()
        }).await?;

        rules.print_formatted();

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
        let rule = BankImportRule {
            member_id: self.member_id,
            iban: self.iban,
            split_amount: self.split_amount,
            match_subject: self.match_subject,
        };
        println!();
        rule.print_formatted();
        println!();
        let ok = Confirm::new("Add this rule?").prompt()?;
        if !ok {
            return Ok(());
        }

        let rule = db.insert(rule).await?;

        println!(
            "Created rule for member id {} and IBAN {}",
            rule.member_id,
            rule.iban,
        );
        
        Ok(())
    }
}


#[derive(Args, Debug)]
pub struct IbanUpdate {
    #[clap(short, long)]
    pub member_id: u32,

    #[clap(short, long)]
    pub iban: String,

    #[clap(short, long)]
    pub split_amount: Option<f64>,

    #[clap(short, long)]
    pub match_subject: Option<String>,
}

impl IbanUpdate {
    pub async fn run(self, db: &Connection) -> Result<()> {
        // Get rule
        let rule: BankImportRule = db.retrieve(
            (self.member_id, self.iban)
        ).await?;

        println!();
        rule.print_formatted();
        println!();

        let mut update = rule.clone();
        if let Some(split_amount) = self.split_amount {
            if split_amount == 0.0 {
                update.split_amount = None;
            } else {
                update.split_amount = Some(split_amount);
            }
        }
        if let Some(match_subject) = self.match_subject {
            if match_subject == "" {
                update.match_subject = None;
            } else {
                update.match_subject = Some(match_subject);
            }
        }

        println!("Update:");
        update.print_formatted();
        println!();

        let ok = Confirm::new("Apply this update?").prompt()?;
        if !ok {
            return Ok(());
        }

        db.update(update).await?;

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
        let rule: BankImportRule = db.retrieve(
            (self.member_id, self.iban)
        ).await?;

        println!();
        rule.print_formatted();
        println!();

        let ok = Confirm::new("Delete this rule?").prompt()?;
        if !ok {
            return Ok(());
        }

        db.delete(rule).await?;

        Ok(())
    }
}
