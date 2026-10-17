use crate::cli::history::range::HistoryRangeArgs;
use clap::Args;

#[derive(Args, Debug)]
pub struct StatusHistoryArgs {
    #[command(flatten)]
    pub history_range: HistoryRangeArgs,
}
