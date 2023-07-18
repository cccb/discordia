use clap::{Parser, Subcommand};
use anyhow::Result;

use eris_db::Connection;

use crate::commands::{Accounting, Bank, Members};

#[derive(Parser, Debug)]
#[clap(name = "eris", version=env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    #[clap(long, default_value = "members.sqlite3")]
    pub members_db: String,

    #[clap(subcommand)]
    pub command: Command,
}

impl Cli {
    pub fn init() -> Self {
        Self::parse()
    }

    pub async fn run(self, db: &Connection) -> Result<()> {
        match self.command {
            Command::Members(cmd) => cmd.run(&db).await,
            Command::Accounting(cmd) => cmd.run(&db).await,
            Command::Bank(cmd) => cmd.run(&db).await,
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum Command {
    #[clap(subcommand, name = "members")]
    /// Manage members
    Members(Members),

    #[clap(subcommand, name = "accounts")]
    /// Calculate account balances and membershipt fees
    Accounting(Accounting),

    #[clap(subcommand, name = "bank")]
    /// Import bank transactions and manage IBAN rules
    Bank(Bank),
}
