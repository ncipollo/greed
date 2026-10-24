use crate::cli::history::quotes::QuotesArgs;
use crate::cli::history::status::StatusHistoryArgs;
use clap::{Args, Subcommand};

mod output_format;
mod quotes;
mod range;
mod status;

#[derive(Args, Debug)]
pub struct HistoryArgs {
    #[command(subcommand)]
    pub command: HistoryCommand,
}

#[derive(Subcommand, Debug)]
pub enum HistoryCommand {
    #[command(about = "Fetch historic portfolio status")]
    Status(StatusHistoryArgs),
    #[command(about = "Fetch historic stock quotes")]
    Quotes(QuotesArgs),
}
