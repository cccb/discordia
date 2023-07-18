
use anyhow::Result;

use eris_db::Connection;
use eris_cli::cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::init();

    let conn = Connection::open(&cli.members_db).await?;
    cli.run(&conn).await?;

    Ok(())
}

