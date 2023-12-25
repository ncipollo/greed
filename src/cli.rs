pub mod analyze;
pub mod quote;
pub mod run;

use crate::cli::analyze::AnalyzeArgs;
use crate::cli::quote::QuoteArgs;
use crate::cli::run::RunCommandArgs;
use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "greed", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(about = "Run the main greed loop")]
    Run(RunCommandArgs),
    #[command(about = "Analyze stocks")]
    Analyze(AnalyzeArgs),
    #[command(about = "Fetch quote")]
    Quote(QuoteArgs),
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
