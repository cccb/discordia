use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(default_value = "members.sqlite3")]
    pub db: String,
}

/// Parse cli args shorthand
pub fn parse() -> Args {
    Args::parse()
}
