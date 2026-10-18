use crate::cli::history::quote_history::QuoteHistoryArgs;
use crate::cli::history::status::StatusHistoryArgs;
use clap::{Args, Subcommand};

mod output_format;
mod quote_history;
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
    QuoteHistory(QuoteHistoryArgs),
}
