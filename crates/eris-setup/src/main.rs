use anyhow::Result;

use clap::{Subcommand, Parser};

use eris_db::{connection, schema};

#[derive(Parser, Debug)]
#[clap(name="eris-setup")]
struct Cli {
    #[clap(default_value="members.sqlite3")]
    pub members_db: String,

    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command{
    Init,
}

/// Initialize the database
async fn db_init(filename: &str) -> Result<()> {
    let conn = connection::open(&filename).await?;
    schema::install(&conn).await?;

    Ok(())
}


#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse(); 
    match cli.command {
        Command::Init => db_init(&cli.members_db).await?,
    }
    Ok(())
}
