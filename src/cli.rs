use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "greed", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(about = "Run the main greed loop")]
    Run,
    #[command(about = "Fetch quote")]
    Quote(DebugOptions),
    #[command(about = "Test Alpaca")]
    TestAlpaca
}

#[derive(Args, Debug)]
pub struct DebugOptions {
    #[arg(value_name = "investing platform")]
    pub platform: Option<String>
}