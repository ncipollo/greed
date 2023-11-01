pub mod run;

use clap::{Args, Parser, Subcommand};
use crate::cli::run::RunCommandArgs;

#[derive(Debug, Parser)]
#[command(name = "greed", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(about = "Run the main greed loop")]
    Run(RunCommandArgs),
    #[command(about = "Fetch quote")]
    Quote(DebugArgs),
    #[command(about = "Test Alpaca")]
    TestAlpaca
}

#[derive(Args, Debug)]
pub struct DebugArgs {
    #[arg(value_name = "investing platform")]
    pub platform: Option<String>
}