
use anyhow::Result;

use eris_cli::cli::Cli;

fn main() -> Result<()> {
    let cli = Cli::init();

    println!("CLI: {:?}", cli);

    Ok(())
}

