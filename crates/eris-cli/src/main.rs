
use anyhow::Result;

use eris_db::Connection;
use eris_cli::cli::{Command, Cli};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::init();

    let conn = Connection::open(&cli.members_db).await?;
    match cli.command {
        Command::Members(cmd) => cmd.run(&conn).await?,
        Command::Accounting(cmd) => cmd.run(&conn).await?,
        Command::Transactions(cmd) => cmd.run(&conn).await?,
    };

    Ok(())
}

