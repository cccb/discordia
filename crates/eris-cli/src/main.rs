
use anyhow::Result;

use eris_db::Connection;
use eris_cli::cli::{Command, Cli};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::init();

    let conn = Connection::open(&cli.members_db).await?;
    match cli.command {
        Command::List(cmd) => cmd.run(&conn).await,
        Command::Add(cmd) => cmd.run(&conn).await,
        Command::Delete(cmd) => cmd.run(&conn).await,
        Command::Update(cmd) => cmd.run(&conn).await,
        Command::Calculate(cmd) => cmd.run(&conn).await,
    }?;

    Ok(())
}

