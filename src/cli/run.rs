use clap::Args;
use greed::run::GreedRunnerArgs;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct RunCommandArgs {
    /// Path to the greed configuration toml file
    #[arg(value_name = "PATH")]
    config_path: PathBuf,
    /// Override the path to the DuckDB database file. Defaults to "greed.duckdb" next to the config file.
    #[arg(long, value_name = "PATH")]
    db_path: Option<PathBuf>,
    /// Use a simulated financial platform instead of a live account.
    #[arg(short = 's', long)]
    is_simulated: bool,
    /// Persist portfolio status and stock quote history to the local DuckDB database.
    #[arg(long)]
    record_history: bool,
}

impl From<RunCommandArgs> for GreedRunnerArgs {
    fn from(value: RunCommandArgs) -> Self {
        Self {
            config_path: value.config_path.clone(),
            db_path: value.db_path,
            is_simulated: value.is_simulated,
            record_history: value.record_history,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::cli::run::RunCommandArgs;
    use greed::run::GreedRunnerArgs;
    use std::path::PathBuf;

    #[test]
    fn runner_args_from() {
        let command_args = RunCommandArgs {
            config_path: PathBuf::from("path"),
            db_path: Some(PathBuf::from("db_path")),
            is_simulated: true,
            record_history: true,
        };
        let runner_args: GreedRunnerArgs = command_args.into();
        let expected = GreedRunnerArgs {
            config_path: PathBuf::from("path"),
            db_path: Some(PathBuf::from("db_path")),
            is_simulated: true,
            record_history: true,
        };
        assert_eq!(runner_args, expected)
    }
}
