use clap::{Parser, Subcommand};

use crate::commands::Members;

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
}

#[derive(Subcommand, Debug)]
pub enum Command {
    #[clap(subcommand, name = "members")]
    Members(Members),
}
