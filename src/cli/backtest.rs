use crate::cli::time_range::TimeRangeArgs;
use clap::Args;
use greed::backtest::{BacktestOptions, TraceFormat, TradingCosts, DEFAULT_STARTING_CASH};
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
    /// A flat fee charged on every simulated fill. Defaults to none.
    #[arg(long, value_name = "DOLLARS", default_value_t = 0.0)]
    pub fee: f64,
    /// How far every fill price moves against you, in basis points (1 bp = 0.01%). Defaults to none.
    #[arg(long, value_name = "BPS", default_value_t = 0.0)]
    pub slippage_bps: f64,
    /// Override the path to the DuckDB database file holding backfilled bars. Defaults to
    /// "greed.duckdb" next to the config file, like `run` and `backfill`.
    #[arg(long, value_name = "PATH")]
    pub db_path: Option<PathBuf>,
    /// Write a trace of every simulated trade to this file. The format comes from the extension:
    /// `.json`, `.toml` or `.html` (a self-contained page you can open offline).
    #[arg(long, value_name = "PATH", value_parser = parse_trace_path)]
    pub trace: Option<PathBuf>,
    /// Also show every simulated interval and the strategies' own logging. Progress is printed to
    /// stderr; the final result goes to stdout.
    #[arg(short = 'v', long)]
    pub verbose: bool,
}

/// Rejects a trace path with an unknown extension while arguments are parsed, before any work.
fn parse_trace_path(value: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(value);
    TraceFormat::from_path(&path).map_err(|e| e.to_string())?;
    Ok(path)
}

impl From<BacktestArgs> for BacktestOptions {
    fn from(value: BacktestArgs) -> Self {
        Self {
            config_path: value.config_path,
            time_range: (&value.time_range).into(),
            starting_cash: value.starting_cash,
            costs: TradingCosts {
                fee: value.fee,
                slippage_bps: value.slippage_bps,
            },
            db_path: value.db_path,
            trace: value.trace,
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
    fn parses_a_trace_path() {
        let options: BacktestOptions = parse(&["greed.toml", "--trace", "out.JSON"])
            .expect("parse")
            .into();

        assert_eq!(options.trace, Some(PathBuf::from("out.JSON")));
        assert_eq!(parse(&["greed.toml"]).expect("parse").trace, None);
    }

    #[test]
    fn rejects_a_trace_with_an_unknown_extension_before_running() {
        assert!(parse(&["greed.toml", "--trace", "out.txt"]).is_err());
        assert!(parse(&["greed.toml", "--trace", "out"]).is_err());
    }

    #[test]
    fn costs_default_to_none_and_parse_when_given() {
        let none: BacktestOptions = parse(&["greed.toml"]).expect("parse").into();
        let costs: BacktestOptions = parse(&["greed.toml", "--fee", "1.5", "--slippage-bps", "10"])
            .expect("parse")
            .into();

        assert_eq!(none.costs, greed::backtest::TradingCosts::default());
        assert_eq!(costs.costs.fee, 1.5);
        assert_eq!(costs.costs.slippage_bps, 10.0);
    }

    #[test]
    fn parses_verbose() {
        assert!(parse(&["greed.toml", "-v"]).expect("parse").verbose);
        assert!(!parse(&["greed.toml"]).expect("parse").verbose);
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
