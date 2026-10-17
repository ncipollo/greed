use crate::cli::history::status::StatusHistoryArgs;
use clap::{Args, Subcommand};

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
}
