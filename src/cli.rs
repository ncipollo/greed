pub mod analyze;
pub mod quote;
pub mod run;
mod status;
mod orders;

use crate::cli::analyze::AnalyzeArgs;
use crate::cli::quote::QuoteArgs;
use crate::cli::run::RunCommandArgs;
use clap::{Args, Parser, Subcommand};
use crate::cli::orders::OrdersArgs;

#[derive(Debug, Parser)]
#[command(name = "greed", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(about = "Analyze stocks")]
    Analyze(AnalyzeArgs),
    #[command(about = "Fetch recent orders")]
    Orders(OrdersArgs),
    #[command(about = "Fetch quote")]
    Quote(QuoteArgs),
    #[command(about = "Run the main greed loop")]
    Run(RunCommandArgs),
    #[command(about = "Get your current creed status")]
    Status(status::StatusArgs),
    #[command(about = "prints out completions for the provided shell")]
    Completions {
        #[arg(value_enum)]
        shell: clap_complete_command::Shell,
    },
}

#[derive(Args, Debug)]
pub struct DebugArgs {
    #[arg(value_name = "investing platform")]
    pub platform: Option<String>,
}
