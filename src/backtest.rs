mod market_data;
mod platform;
mod portfolio;

pub use platform::BacktestPlatform;

use crate::asset::AssetSymbol;
use crate::backfill::{target_from_config, CoverageChecker};
use crate::clock::SimulatedClock;
use crate::config::strategy::StrategyConfig;
use crate::config::Config;
use crate::db;
use crate::error::GreedError;
use crate::platform::bar::time_frame::TimeFrame;
use crate::strategy::reader::read_config_from_path;
use crate::time_range::TimeRangeSpec;
use chrono::{DateTime, Utc};
use market_data::MarketData;
use std::fmt::{Display, Formatter};
use std::ops::Range;
use std::path::PathBuf;
use std::sync::Arc;

/// Starting buying power when `--starting-cash` isn't given.
pub const DEFAULT_STARTING_CASH: f64 = 100_000.0;

/// Everything `greed backtest` needs, as parsed from the command line.
#[derive(Debug)]
pub struct BacktestOptions {
    pub config_path: PathBuf,
    pub time_range: TimeRangeSpec,
    pub starting_cash: f64,
    /// Overrides the DuckDB path; defaults next to the config like `run` and `backfill`.
    pub db_path: Option<PathBuf>,
}

/// A backtest that is ready to simulate: config validated, bars loaded into
/// memory, and a simulated platform wired to a simulated clock set to the
/// start of the range.
pub struct PreparedBacktest {
    pub config: Config,
    pub config_path: PathBuf,
    pub range: Range<DateTime<Utc>>,
    pub symbols: Vec<AssetSymbol>,
    pub platform: Arc<BacktestPlatform>,
    pub clock: Arc<SimulatedClock>,
    pub starting_cash: f64,
    pub bars_loaded: usize,
    /// False when the database lacks minute coverage for some symbol over the
    /// range, so the simulation would have holes. `greed backfill` fills them.
    pub data_complete: bool,
}

impl Display for PreparedBacktest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let format = "%Y-%m-%d %H:%M UTC";
        writeln!(
            f,
            "Backtest {} to {}",
            self.range.start.format(format),
            self.range.end.format(format)
        )?;
        writeln!(
            f,
            "  symbols: {}",
            self.symbols
                .iter()
                .map(|s| &s.symbol)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        writeln!(f, "  starting cash: ${:.2}", self.starting_cash)?;
        write!(f, "  minute bars loaded: {}", self.bars_loaded)?;
        if !self.data_complete {
            write!(
                f,
                "\n  warning: stored data doesn't cover the whole range; run `greed backfill --config {}` first",
                self.config_path.display()
            )?;
        }
        Ok(())
    }
}

/// Runs `greed backtest`: validates the config, loads historical bars and
/// builds the simulated platform.
pub async fn run(options: BacktestOptions) -> Result<PreparedBacktest, GreedError> {
    prepare(options, Utc::now()).await
}

pub(crate) async fn prepare(
    options: BacktestOptions,
    now: DateTime<Utc>,
) -> Result<PreparedBacktest, GreedError> {
    if options.starting_cash <= 0.0 || !options.starting_cash.is_finite() {
        return Err(GreedError::new("starting cash must be a positive amount"));
    }
    let config = read_config_from_path(&options.config_path).await?;
    reject_unsimulatable(&config)?;
    let range = options.time_range.resolve(now)?;
    let target = target_from_config(&options.config_path).await?;
    if target.symbols.is_empty() {
        return Err(GreedError::new("the config trades no symbols to backtest"));
    }
    let db_path = db::path_for_run(&options.config_path, options.db_path.as_deref());
    let connection = db::open_read_only_with_retry(&db_path)
        .await?
        .ok_or_else(|| {
            GreedError::new(&format!(
                "no database at {}; run `greed backfill --config {}` first",
                db_path.display(),
                options.config_path.display()
            ))
        })?;
    let market = MarketData::load(&connection, &target.symbols, &range)?;
    let plan = CoverageChecker::plan(
        &db_path,
        &target.symbols,
        &TimeFrame::OneMinute,
        range.clone(),
    )
    .await?;
    let bars_loaded = market.bar_count();
    let clock = Arc::new(SimulatedClock::new(range.start));
    let platform = Arc::new(BacktestPlatform::new(
        clock.clone(),
        market,
        options.starting_cash,
    ));
    Ok(PreparedBacktest {
        config,
        config_path: options.config_path,
        range,
        symbols: target.symbols,
        platform,
        clock,
        starting_cash: options.starting_cash,
        bars_loaded,
        data_complete: plan.is_complete(),
    })
}

/// Agent strategies call an LLM and the web, which is non-deterministic and
/// needs the network, so they can't be replayed.
fn reject_unsimulatable(config: &Config) -> Result<(), GreedError> {
    for strategy in &config.strategies {
        if let StrategyConfig::Agent {
            agent_path,
            properties,
        } = strategy
        {
            let name = if properties.name.is_empty() {
                agent_path
            } else {
                &properties.name
            };
            return Err(GreedError::new(&format!(
                "agent strategy '{name}' can't be backtested: agents call an LLM and the web, \
                 which can't be replayed. Remove it from the config to backtest the rest"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::db::bootstrap_at;
    use crate::fixture;
    use crate::platform::bar::Bar;
    use chrono::TimeZone;
    use uuid::Uuid;

    fn options(config: &str, db_path: Option<PathBuf>) -> BacktestOptions {
        BacktestOptions {
            config_path: fixture::path(config),
            time_range: TimeRangeSpec {
                from: Some("2026-01-05".to_string()),
                to: Some("2026-01-06".to_string()),
                ..Default::default()
            },
            starting_cash: DEFAULT_STARTING_CASH,
            db_path,
        }
    }

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap()
    }

    fn temp_db() -> PathBuf {
        let path = std::env::temp_dir().join(format!("greed_backtest_{}.duckdb", Uuid::new_v4()));
        bootstrap_at(&path).unwrap();
        path
    }

    #[test]
    fn agent_strategies_are_rejected_by_name() {
        let config = Config {
            strategies: vec![StrategyConfig::Agent {
                agent_path: "agents/trader.md".to_string(),
                properties: crate::config::strategy::StrategyProperties {
                    name: "momentum-bot".to_string(),
                    ..Default::default()
                },
            }],
            ..Default::default()
        };

        let error = reject_unsimulatable(&config).unwrap_err().to_string();

        assert!(error.contains("momentum-bot"), "{error}");
        assert!(error.contains("can't be backtested"), "{error}");
    }

    #[test]
    fn config_file_strategies_are_allowed() {
        let config = Config {
            strategies: vec![StrategyConfig::default()],
            ..Default::default()
        };

        assert!(reject_unsimulatable(&config).is_ok());
    }

    #[tokio::test]
    async fn a_missing_database_is_an_error_pointing_at_backfill() {
        let missing = std::env::temp_dir().join(format!("greed_nope_{}.duckdb", Uuid::new_v4()));

        let result = prepare(options("config_single_tactic.toml", Some(missing)), now()).await;

        let error = result.err().expect("should fail").to_string();
        assert!(error.contains("greed backfill"), "{error}");
    }

    #[tokio::test]
    async fn rejects_non_positive_starting_cash() {
        let mut opts = options("config_single_tactic.toml", None);
        opts.starting_cash = 0.0;

        assert!(prepare(opts, now()).await.is_err());
    }

    #[tokio::test]
    async fn loads_bars_and_starts_the_clock_at_the_range_start() {
        let path = temp_db();
        let mut connection = db::open_for_write(&path).unwrap();
        let bars = vec![Bar {
            timestamp: Utc.with_ymd_and_hms(2026, 1, 5, 14, 30, 0).unwrap(),
            open: 1.0,
            high: 1.0,
            low: 1.0,
            close: 1.0,
            volume: 1,
        }];
        let window = Utc.with_ymd_and_hms(2026, 1, 5, 0, 0, 0).unwrap()
            ..Utc.with_ymd_and_hms(2026, 1, 7, 0, 0, 0).unwrap();
        db::stock_bars::insert_chunk(
            &mut connection,
            &AssetSymbol::new("VTI"),
            &TimeFrame::OneMinute,
            &window,
            &bars,
        )
        .unwrap();
        drop(connection);

        let prepared = prepare(
            options("config_single_tactic.toml", Some(path.clone())),
            now(),
        )
        .await
        .unwrap();

        assert_eq!(prepared.bars_loaded, 1);
        assert!(prepared.data_complete);
        assert_eq!(prepared.symbols, vec![AssetSymbol::new("VTI")]);
        use crate::clock::Clock;
        assert_eq!(prepared.clock.now(), prepared.range.start);
        std::fs::remove_file(&path).ok();
    }

    #[tokio::test]
    async fn incomplete_coverage_is_flagged() {
        let path = temp_db();

        let prepared = prepare(
            options("config_single_tactic.toml", Some(path.clone())),
            now(),
        )
        .await
        .unwrap();

        assert!(!prepared.data_complete);
        assert!(prepared.to_string().contains("greed backfill"));
        std::fs::remove_file(&path).ok();
    }
}
