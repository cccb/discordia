
use clap::{Parser, Subcommand};

use crate::commands::{
    AddMember,
    UpdateMember,
    CalculateAccounts,
};

#[derive(Parser, Debug)]
#[clap(name = "eris", version = "1.0")]
pub struct Cli {
    #[clap(default_value = "members.sqlite3")]
    pub members_db: String,

    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    #[clap(name = "add")]
    Add(AddMember),
    #[clap(name = "update")]
    Update(UpdateMember),

    #[clap(name = "calculate_accounts")]
    Calculate(CalculateAccounts),
}

impl Cli {
    pub fn init() -> Self {
        Self::parse()
    }
}

