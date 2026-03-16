use clap::{Args, ValueEnum};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Type of config to generate (greed, strategy, agent)
    #[arg(value_name = "TYPE", value_enum)]
    pub config_type: InitConfigType,

    /// Path to write the template file (defaults to current directory)
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum InitConfigType {
    Greed,
    Strategy,
    Agent,
}
