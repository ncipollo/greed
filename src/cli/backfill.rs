use crate::cli::time_range::TimeRangeArgs;
use clap::{ArgGroup, Args};
use greed::asset::AssetSymbol;
use greed::backfill::{BackfillRequest, BackfillTarget};
use greed::config::platform::PlatformType;
use greed::platform::args::PlatformArgs;
use std::path::PathBuf;

#[derive(Args, Debug)]
#[command(group(ArgGroup::new("target").required(true).args(["symbols", "config"])))]
pub struct BackfillArgs {
    /// Ticker symbols to backfill, e.g. `VTI SPY`.
    #[arg(value_name = "SYMBOLS")]
    pub symbols: Vec<AssetSymbol>,
    /// Backfill every symbol the given greed config trades instead.
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,
    #[command(flatten)]
    pub time_range: TimeRangeArgs,
    /// Override the path to the DuckDB database file. Defaults to "greed.duckdb" next to the
    /// config file, or in the current directory when backfilling symbols.
    #[arg(long, value_name = "PATH")]
    pub db_path: Option<PathBuf>,
    /// Use a simulated financial platform instead of a live account.
    #[arg(short = 's', long)]
    pub is_simulated: bool,
    /// The platform to fetch from when backfilling symbols; a config names its own.
    #[arg(short = 'p', long, conflicts_with = "config")]
    pub platform_type: Option<PlatformType>,
    /// Requests per minute to pace to. Defaults to Alpaca's free-tier limit (200); raise it if
    /// your data plan allows more.
    #[arg(long, value_name = "N", value_parser = clap::value_parser!(u32).range(1..))]
    pub requests_per_minute: Option<u32>,
}

impl From<BackfillArgs> for BackfillRequest {
    fn from(value: BackfillArgs) -> Self {
        let target = match value.config {
            Some(config_path) => BackfillTarget::Config(config_path),
            None => BackfillTarget::Symbols(value.symbols),
        };
        Self {
            target,
            time_range: (&value.time_range).into(),
            db_path: value.db_path,
            platform_type: value.platform_type.unwrap_or_default(),
            platform_args: PlatformArgs {
                is_simulated: value.is_simulated,
                requests_per_minute: value.requests_per_minute,
            },
        }
    }
}

#[cfg(test)]
mod test {
    use crate::cli::{Cli, Command};
    use clap::Parser;
    use greed::asset::AssetSymbol;
    use greed::backfill::{BackfillRequest, BackfillTarget};
    use greed::time_range::TimeRangeSpec;
    use std::path::PathBuf;

    fn parse(args: &[&str]) -> Result<super::BackfillArgs, clap::Error> {
        let cli = Cli::try_parse_from(["greed", "backfill"].iter().chain(args))?;
        let Command::Backfill(args) = cli.command else {
            panic!("expected Backfill command")
        };
        Ok(args)
    }

    #[test]
    fn parses_symbols() {
        let args = parse(&["VTI", "$SPY"]).expect("should parse");

        let request: BackfillRequest = args.into();

        assert_eq!(
            request.target,
            BackfillTarget::Symbols(vec![AssetSymbol::new("VTI"), AssetSymbol::new("SPY")])
        );
        assert_eq!(request.time_range, TimeRangeSpec::default());
        assert_eq!(request.platform_args.requests_per_minute, None);
        assert!(!request.platform_args.is_simulated);
    }

    #[test]
    fn parses_config() {
        let args = parse(&["--config", "greed.toml"]).expect("should parse");

        let request: BackfillRequest = args.into();

        assert_eq!(
            request.target,
            BackfillTarget::Config(PathBuf::from("greed.toml"))
        );
    }

    #[test]
    fn requires_symbols_or_config() {
        assert!(parse(&[]).is_err());
    }

    #[test]
    fn rejects_symbols_together_with_config() {
        assert!(parse(&["VTI", "--config", "greed.toml"]).is_err());
    }

    #[test]
    fn rejects_a_platform_type_together_with_config() {
        assert!(parse(&["--config", "greed.toml", "-p", "alpaca"]).is_err());
    }

    #[test]
    fn passes_time_range_options_through() {
        let args = parse(&["VTI", "--from", "2026-01-01", "--to", "2026-02-01"]).expect("parse");

        let request: BackfillRequest = args.into();

        let expected = TimeRangeSpec {
            from: Some("2026-01-01".to_string()),
            to: Some("2026-02-01".to_string()),
            ..Default::default()
        };
        assert_eq!(request.time_range, expected);
    }

    #[test]
    fn passes_platform_options_through() {
        let args = parse(&["VTI", "-s", "--requests-per-minute", "10000"]).expect("parse");

        let request: BackfillRequest = args.into();

        assert!(request.platform_args.is_simulated);
        assert_eq!(request.platform_args.requests_per_minute, Some(10_000));
    }

    #[test]
    fn rejects_a_zero_request_rate() {
        assert!(parse(&["VTI", "--requests-per-minute", "0"]).is_err());
    }

    #[test]
    fn parses_db_path() {
        let args = parse(&["VTI", "--db-path", "/tmp/x.duckdb"]).expect("parse");
        let request: BackfillRequest = args.into();
        assert_eq!(request.db_path, Some(PathBuf::from("/tmp/x.duckdb")));
    }
}
