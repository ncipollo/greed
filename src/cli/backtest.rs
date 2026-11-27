use crate::cli::time_range::TimeRangeArgs;
use clap::Args;
use greed::backtest::{BacktestOptions, DEFAULT_STARTING_CASH};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct BacktestArgs {
    /// The greed config to simulate.
    #[arg(value_name = "CONFIG")]
    pub config_path: PathBuf,
    #[command(flatten)]
    pub time_range: TimeRangeArgs,
    /// Starting buying power for the simulated account.
    #[arg(long, value_name = "DOLLARS", default_value_t = DEFAULT_STARTING_CASH)]
    pub starting_cash: f64,
    /// Override the path to the DuckDB database file holding backfilled bars. Defaults to
    /// "greed.duckdb" next to the config file, like `run` and `backfill`.
    #[arg(long, value_name = "PATH")]
    pub db_path: Option<PathBuf>,
}

impl From<BacktestArgs> for BacktestOptions {
    fn from(value: BacktestArgs) -> Self {
        Self {
            config_path: value.config_path,
            time_range: (&value.time_range).into(),
            starting_cash: value.starting_cash,
            db_path: value.db_path,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::cli::{Cli, Command};
    use clap::Parser;
    use greed::backtest::BacktestOptions;
    use greed::time_range::TimeRangeSpec;
    use std::path::PathBuf;

    fn parse(args: &[&str]) -> Result<super::BacktestArgs, clap::Error> {
        let cli = Cli::try_parse_from(["greed", "backtest"].iter().chain(args))?;
        let Command::Backtest(args) = cli.command else {
            panic!("expected Backtest command")
        };
        Ok(args)
    }

    #[test]
    fn defaults_to_a_year_and_one_hundred_thousand_dollars() {
        let options: BacktestOptions = parse(&["greed.toml"]).expect("parse").into();

        assert_eq!(options.config_path, PathBuf::from("greed.toml"));
        assert_eq!(options.time_range, TimeRangeSpec::default());
        assert_eq!(options.starting_cash, 100_000.0);
        assert_eq!(options.db_path, None);
    }

    #[test]
    fn requires_a_config() {
        assert!(parse(&[]).is_err());
    }

    #[test]
    fn parses_starting_cash_and_db_path() {
        let options: BacktestOptions = parse(&[
            "greed.toml",
            "--starting-cash",
            "2500.5",
            "--db-path",
            "/tmp/x.duckdb",
        ])
        .expect("parse")
        .into();

        assert_eq!(options.starting_cash, 2500.5);
        assert_eq!(options.db_path, Some(PathBuf::from("/tmp/x.duckdb")));
    }

    #[test]
    fn accepts_the_shared_time_range_options() {
        let options: BacktestOptions = parse(&["greed.toml", "--months", "6"])
            .expect("parse")
            .into();

        assert_eq!(options.time_range.months, Some(6));
        assert!(parse(&["greed.toml", "--from", "2026-01-01", "--days", "3"]).is_err());
    }
}
