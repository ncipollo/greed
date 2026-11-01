use clap::Args;
use greed::run::GreedRunnerArgs;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct CommonRunArgs {
    /// Path to the greed configuration toml file
    #[arg(value_name = "PATH")]
    pub config_path: PathBuf,
    /// Override the path to the DuckDB database file. Defaults to "greed.duckdb" next to the config file.
    #[arg(long, value_name = "PATH")]
    pub db_path: Option<PathBuf>,
    /// Use a simulated financial platform instead of a live account.
    #[arg(short = 's', long)]
    pub is_simulated: bool,
    /// Persist portfolio status and stock quote history to the local DuckDB database.
    #[arg(long)]
    pub record_history: bool,
}

impl From<CommonRunArgs> for GreedRunnerArgs {
    fn from(value: CommonRunArgs) -> Self {
        Self {
            config_path: value.config_path,
            db_path: value.db_path,
            is_simulated: value.is_simulated,
            record_history: value.record_history,
        }
    }
}

#[derive(Args, Debug)]
pub struct RunCommandArgs {
    #[command(flatten)]
    pub common: CommonRunArgs,
}

impl From<RunCommandArgs> for GreedRunnerArgs {
    fn from(value: RunCommandArgs) -> Self {
        value.common.into()
    }
}

#[cfg(test)]
mod test {
    use crate::cli::{Cli, Command};
    use clap::Parser;
    use greed::run::GreedRunnerArgs;
    use std::path::PathBuf;

    #[test]
    fn runner_args_from() {
        let cli = Cli::try_parse_from(["greed", "run", "-s", "--record-history", "path"])
            .expect("should parse");
        let Command::Run(args) = cli.command else {
            panic!("expected Run command")
        };
        let runner_args: GreedRunnerArgs = args.into();
        let expected = GreedRunnerArgs {
            config_path: PathBuf::from("path"),
            db_path: None,
            is_simulated: true,
            record_history: true,
        };
        assert_eq!(runner_args, expected)
    }

    #[test]
    fn parses_run_with_common_flags() {
        let cli = Cli::try_parse_from([
            "greed",
            "run",
            "-s",
            "--record-history",
            "--db-path",
            "db_path",
            "cfg.toml",
        ])
        .expect("should parse");
        let Command::Run(args) = cli.command else {
            panic!("expected Run command")
        };
        assert!(args.common.is_simulated);
        assert!(args.common.record_history);
        assert_eq!(args.common.db_path, Some(PathBuf::from("db_path")));
        assert_eq!(args.common.config_path, PathBuf::from("cfg.toml"));
    }
}
