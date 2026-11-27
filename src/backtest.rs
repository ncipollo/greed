//! Backtesting: replay a greed config against stored historical bars.
//!
//! Two guarantees hold by construction, not convention:
//!
//! 1. **No network during simulation.** The only code that may build a live platform is the
//!    preflight backfill ([`preflight::ensure_data`]), which does so through a closure and drops it
//!    before returning. [`simulate`] takes a [`PreparedBacktest`] whose platform is the concrete
//!    [`BacktestPlatform`], so a live platform can't be passed in. Agent strategies, which need an
//!    LLM and the web, are rejected up front.
//! 2. **Nothing is recorded.** Simulation uses `GreedRunner::for_simulation`, which has no database
//!    path, no history recording and no bar recorder. Bars are loaded
//!    through a read-only connection that is closed before the first tick. Only the preflight
//!    backfill writes (`stock_bars`, `backfill_coverage`).

mod engine;
mod market_data;
mod platform;
mod portfolio;
mod preflight;
mod trace;

pub use engine::simulate;
pub use platform::BacktestPlatform;
pub use trace::{BacktestTrace, TraceFormat};

use crate::asset::AssetSymbol;
use crate::backfill::target_from_config;
use crate::clock::SimulatedClock;
use crate::config::strategy::StrategyConfig;
use crate::config::Config;
use crate::db;
use crate::error::GreedError;
use crate::platform::args::PlatformArgs;
use crate::platform::FinancialPlatform;
use crate::progress::{Phase, Progress};
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
    /// Where to write a trace of every simulated trade (`.json`, `.toml` or `.html`).
    pub trace: Option<PathBuf>,
}

/// A backtest that is ready to simulate: config validated, data backfilled and loaded into
/// memory, and a simulated platform wired to a simulated clock.
pub struct PreparedBacktest {
    pub config: Config,
    pub config_path: PathBuf,
    pub range: Range<DateTime<Utc>>,
    pub symbols: Vec<AssetSymbol>,
    pub platform: Arc<BacktestPlatform>,
    pub clock: Arc<SimulatedClock>,
    pub starting_cash: f64,
    pub bars_loaded: usize,
    /// Where to write the trace, and what collects it, when one was asked for.
    pub trace: Option<(PathBuf, trace::TraceRecorder)>,
}

/// How a simulated run ended.
#[derive(Clone, Debug, PartialEq)]
pub struct BacktestResult {
    pub range: Range<DateTime<Utc>>,
    pub starting_cash: f64,
    /// Cash plus open positions valued at the last close at or before the end of the range.
    pub final_value: f64,
    pub return_percent: f64,
    pub trade_count: usize,
}

impl BacktestResult {
    pub fn new(
        range: Range<DateTime<Utc>>,
        starting_cash: f64,
        final_value: f64,
        trade_count: usize,
    ) -> Self {
        Self {
            range,
            starting_cash,
            final_value,
            return_percent: (final_value - starting_cash) / starting_cash * 100.0,
            trade_count,
        }
    }
}

impl Display for BacktestResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let format = "%Y-%m-%d %H:%M UTC";
        writeln!(
            f,
            "Backtest {} to {}",
            self.range.start.format(format),
            self.range.end.format(format)
        )?;
        writeln!(f, "  starting cash: ${:.2}", self.starting_cash)?;
        writeln!(f, "  trades:        {}", self.trade_count)?;
        writeln!(f, "  return:        {:+.2}%", self.return_percent)?;
        write!(f, "Final value: ${:.2}", self.final_value)
    }
}

/// Runs `greed backtest`: validates the config, makes sure the stored data covers the range
/// (backfilling from the live platform if not), then simulates.
pub async fn run(
    options: BacktestOptions,
    progress: &Progress,
) -> Result<BacktestResult, GreedError> {
    let prepared = prepare(options, Utc::now(), live_platform, progress).await?;
    simulate(prepared, progress).await
}

fn live_platform(config: &Config) -> Result<Arc<dyn FinancialPlatform>, GreedError> {
    crate::platform::for_type(&config.platform, PlatformArgs::default())
}

/// Everything before the simulation: validation, the up-front backfill (the only phase that may
/// touch the network or write to the database) and loading bars into memory.
pub(crate) async fn prepare<F>(
    options: BacktestOptions,
    now: DateTime<Utc>,
    make_platform: F,
    progress: &Progress,
) -> Result<PreparedBacktest, GreedError>
where
    F: FnOnce(&Config) -> Result<Arc<dyn FinancialPlatform>, GreedError>,
{
    if options.starting_cash <= 0.0 || !options.starting_cash.is_finite() {
        return Err(GreedError::new("starting cash must be a positive amount"));
    }
    let config = read_config_from_path(&options.config_path).await?;
    reject_unsimulatable(&config)?;
    if config.interval == 0 {
        return Err(GreedError::new(
            "the config's interval must be at least 1 second",
        ));
    }
    if let Some(path) = &options.trace {
        TraceFormat::from_path(path)?;
    }
    let range = options.time_range.resolve(now)?;
    let target = target_from_config(&options.config_path).await?;
    if target.symbols.is_empty() {
        return Err(GreedError::new("the config trades no symbols to backtest"));
    }
    let db_path = db::path_for_run(&options.config_path, options.db_path.as_deref());
    let needed = preflight::ensure_data(
        &db_path,
        &target.symbols,
        &range,
        now,
        || make_platform(&config),
        progress,
    )
    .await?;
    progress.phase(Phase::LoadingData);
    let market = {
        let connection = db::open_read_only_with_retry(&db_path)
            .await?
            .ok_or_else(|| GreedError::new("the database disappeared after the backfill"))?;
        MarketData::load(&connection, &target.symbols, &needed)?
    };
    let bars_loaded = market.bar_count();
    let clock = Arc::new(SimulatedClock::new(range.start));
    let trace = options
        .trace
        .map(|path| (path, trace::TraceRecorder::new()));
    let mut platform = BacktestPlatform::new(clock.clone(), market, options.starting_cash);
    if let Some((_, recorder)) = &trace {
        platform = platform.with_trace(recorder.clone());
    }
    let platform = Arc::new(platform);
    Ok(PreparedBacktest {
        config,
        config_path: options.config_path,
        range,
        symbols: target.symbols,
        platform,
        clock,
        starting_cash: options.starting_cash,
        bars_loaded,
        trace,
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
    use crate::clock::Clock;
    use crate::db::{bootstrap_at, stock_bars};
    use crate::fixture;
    use crate::platform::bar::time_frame::TimeFrame;
    use crate::platform::bar::Bar;
    use crate::platform::MockPlatform;
    use chrono::{Duration, TimeZone};
    use duckdb::Connection;
    use std::path::Path;
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
            trace: None,
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

    fn no_platform(_: &Config) -> Result<Arc<dyn FinancialPlatform>, GreedError> {
        panic!("this test must not need a live platform")
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
    async fn rejects_non_positive_starting_cash() {
        let mut opts = options("config_single_tactic.toml", None);
        opts.starting_cash = 0.0;

        assert!(prepare(opts, now(), no_platform, &Progress::quiet())
            .await
            .is_err());
    }

    #[tokio::test]
    async fn a_missing_database_is_backfilled_before_loading() {
        let path = std::env::temp_dir().join(format!("greed_nope_{}.duckdb", Uuid::new_v4()));
        let platform = Arc::new(MockPlatform::new());
        let for_factory = platform.clone();

        let prepared = prepare(
            options("config_single_tactic.toml", Some(path.clone())),
            now(),
            move |_| Ok(for_factory as Arc<dyn FinancialPlatform>),
            &Progress::quiet(),
        )
        .await
        .unwrap();

        assert!(!platform.bar_requests().is_empty());
        assert_eq!(prepared.bars_loaded, 0);
        assert!(path.exists());
        std::fs::remove_file(&path).ok();
    }

    #[tokio::test]
    async fn stored_data_is_loaded_without_a_live_platform() {
        let path = temp_db();
        let start = Utc.with_ymd_and_hms(2026, 1, 5, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 1, 7, 0, 0, 0).unwrap();
        let bars = vec![Bar {
            timestamp: Utc.with_ymd_and_hms(2026, 1, 5, 14, 30, 0).unwrap(),
            open: 1.0,
            high: 1.0,
            low: 1.0,
            close: 1.0,
            volume: 1,
        }];
        let mut connection = db::open_for_write(&path).unwrap();
        stock_bars::insert_chunk(
            &mut connection,
            &AssetSymbol::new("VTI"),
            &TimeFrame::OneMinute,
            &((start - Duration::days(35))..end),
            &bars,
        )
        .unwrap();
        drop(connection);

        let prepared = prepare(
            options("config_single_tactic.toml", Some(path.clone())),
            now(),
            no_platform,
            &Progress::quiet(),
        )
        .await
        .unwrap();

        assert_eq!(prepared.bars_loaded, 1);
        assert_eq!(prepared.symbols, vec![AssetSymbol::new("VTI")]);
        assert_eq!(prepared.clock.now(), prepared.range.start);
        std::fs::remove_file(&path).ok();
    }

    /// A config that buys 10% of equity in VTI on every tick and never sells.
    fn always_buy_config() -> PathBuf {
        let path = std::env::temp_dir().join(format!("greed_cfg_{}.toml", Uuid::new_v4()));
        std::fs::write(
            &path,
            r#"
interval = 60
platform = "alpaca"

[[tactics]]
name = "always"

[tactics.buy]
for = { stock = "$VTI" }
when = { always = true }
do = { buy_percent = 10 }

[tactics.sell]
for = { stock = "$VTI" }
when = { never = true }
do = { sell_all = true }
"#,
        )
        .unwrap();
        path
    }

    /// Minute bars 14:30..=15:30 on Monday 2026-01-05 at $100, then $110 from 15:00.
    fn seed_bars(db_path: &Path, range_start: DateTime<Utc>, range_end: DateTime<Utc>) {
        let first = Utc.with_ymd_and_hms(2026, 1, 5, 14, 30, 0).unwrap();
        let switch = Utc.with_ymd_and_hms(2026, 1, 5, 15, 0, 0).unwrap();
        let bars: Vec<Bar> = (0..=60)
            .map(|minute| {
                let timestamp = first + Duration::minutes(minute);
                let price = if timestamp < switch { 100.0 } else { 110.0 };
                Bar {
                    timestamp,
                    open: price,
                    high: price,
                    low: price,
                    close: price,
                    volume: 1,
                }
            })
            .collect();
        let mut connection = db::open_for_write(db_path).unwrap();
        stock_bars::insert_chunk(
            &mut connection,
            &AssetSymbol::new("VTI"),
            &TimeFrame::OneMinute,
            &((range_start - Duration::days(35))..range_end),
            &bars,
        )
        .unwrap();
    }

    fn simulation_options(config_path: PathBuf, db_path: PathBuf) -> BacktestOptions {
        BacktestOptions {
            config_path,
            time_range: TimeRangeSpec {
                from: Some("2026-01-05 14:30".to_string()),
                to: Some("2026-01-05 15:30".to_string()),
                ..Default::default()
            },
            starting_cash: DEFAULT_STARTING_CASH,
            db_path: Some(db_path),
            trace: None,
        }
    }

    fn row_count(path: &PathBuf, table: &str) -> i64 {
        Connection::open(path)
            .unwrap()
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .unwrap()
    }

    #[tokio::test]
    async fn simulation_trades_at_the_simulated_price_and_values_at_the_end() {
        let db_path = temp_db();
        let config_path = always_buy_config();
        let start = Utc.with_ymd_and_hms(2026, 1, 5, 14, 30, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 1, 5, 15, 30, 0).unwrap();
        seed_bars(&db_path, start, end);

        let prepared = prepare(
            simulation_options(config_path.clone(), db_path.clone()),
            now(),
            no_platform,
            &Progress::quiet(),
        )
        .await
        .unwrap();
        let result = simulate(prepared, &Progress::quiet()).await.unwrap();

        // Tick 1: 10% of $100,000 buys 100 shares at $100. When the price rises to $110, equity is
        // $101,000, so the 10% target is $10,100 against a $10,000 cost basis: one more $100 buy.
        // After that the position is on target, so every other tick trades nothing. Everything is
        // valued at $110 at the end: $101,000.
        assert_eq!(result.trade_count, 2);
        assert!(
            (result.final_value - 101_000.0).abs() < 1e-6,
            "{}",
            result.final_value
        );
        assert!((result.return_percent - 1.0).abs() < 1e-9);
        std::fs::remove_file(&db_path).ok();
        std::fs::remove_file(&config_path).ok();
    }

    #[tokio::test]
    async fn the_same_inputs_give_the_same_result() {
        let db_path = temp_db();
        let config_path = always_buy_config();
        let start = Utc.with_ymd_and_hms(2026, 1, 5, 14, 30, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 1, 5, 15, 30, 0).unwrap();
        seed_bars(&db_path, start, end);
        let mut results = Vec::new();

        for _ in 0..2 {
            let prepared = prepare(
                simulation_options(config_path.clone(), db_path.clone()),
                now(),
                no_platform,
                &Progress::quiet(),
            )
            .await
            .unwrap();
            results.push(simulate(prepared, &Progress::quiet()).await.unwrap());
        }

        assert_eq!(results[0], results[1]);
        std::fs::remove_file(&db_path).ok();
        std::fs::remove_file(&config_path).ok();
    }

    #[tokio::test]
    async fn simulation_never_writes_to_the_database() {
        let db_path = temp_db();
        let config_path = always_buy_config();
        let start = Utc.with_ymd_and_hms(2026, 1, 5, 14, 30, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 1, 5, 15, 30, 0).unwrap();
        seed_bars(&db_path, start, end);
        let tables = ["portfolio_status", "stock_bars", "backfill_coverage"];
        let before: Vec<i64> = tables.iter().map(|t| row_count(&db_path, t)).collect();

        let prepared = prepare(
            simulation_options(config_path.clone(), db_path.clone()),
            now(),
            no_platform,
            &Progress::quiet(),
        )
        .await
        .unwrap();
        simulate(prepared, &Progress::quiet()).await.unwrap();

        let after: Vec<i64> = tables.iter().map(|t| row_count(&db_path, t)).collect();
        assert_eq!(before, after);
        assert_eq!(row_count(&db_path, "portfolio_status"), 0);

        std::fs::remove_file(&db_path).ok();
        std::fs::remove_file(&config_path).ok();
    }

    #[tokio::test]
    async fn closed_market_stretches_are_skipped() {
        let db_path = temp_db();
        let config_path = always_buy_config();
        let start = Utc.with_ymd_and_hms(2026, 1, 5, 14, 30, 0).unwrap();
        // Two days long, but bars exist only for one hour on the first day.
        let end = Utc.with_ymd_and_hms(2026, 1, 7, 15, 30, 0).unwrap();
        seed_bars(&db_path, start, end);
        let mut opts = simulation_options(config_path.clone(), db_path.clone());
        opts.time_range.to = Some("2026-01-07 15:30".to_string());

        let prepared = prepare(opts, now(), no_platform, &Progress::quiet())
            .await
            .unwrap();
        let result = simulate(prepared, &Progress::quiet()).await.unwrap();

        // The overnight and second day have no bars, so no extra trades happen and the position
        // is still valued at the last close.
        assert_eq!(result.trade_count, 2);
        assert!(
            (result.final_value - 101_000.0).abs() < 1e-6,
            "{}",
            result.final_value
        );
        std::fs::remove_file(&db_path).ok();
        std::fs::remove_file(&config_path).ok();
    }

    #[tokio::test]
    async fn strategy_file_configs_simulate_without_a_live_platform() {
        let db_path = temp_db();
        let for_factory = Arc::new(MockPlatform::new());

        let prepared = prepare(
            BacktestOptions {
                config_path: fixture::path("config_strategic.toml"),
                time_range: TimeRangeSpec {
                    from: Some("2026-01-05".to_string()),
                    to: Some("2026-01-06".to_string()),
                    ..Default::default()
                },
                starting_cash: DEFAULT_STARTING_CASH,
                db_path: Some(db_path.clone()),
                trace: None,
            },
            now(),
            move |_| Ok(for_factory as Arc<dyn FinancialPlatform>),
            &Progress::quiet(),
        )
        .await
        .unwrap();
        let result = simulate(prepared, &Progress::quiet()).await.unwrap();

        // No stored bars means nothing ever trades, and the account is untouched.
        assert_eq!(result.trade_count, 0);
        assert_eq!(result.final_value, DEFAULT_STARTING_CASH);
        std::fs::remove_file(&db_path).ok();
    }

    #[tokio::test]
    async fn a_trace_records_trades_with_their_tactic_and_daily_equity() {
        let db_path = temp_db();
        let config_path = always_buy_config();
        let trace_path = std::env::temp_dir().join(format!("greed_trace_{}.json", Uuid::new_v4()));
        let start = Utc.with_ymd_and_hms(2026, 1, 5, 14, 30, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 1, 5, 15, 30, 0).unwrap();
        seed_bars(&db_path, start, end);
        let mut opts = simulation_options(config_path.clone(), db_path.clone());
        opts.trace = Some(trace_path.clone());

        let prepared = prepare(opts, now(), no_platform, &Progress::quiet())
            .await
            .unwrap();
        let result = simulate(prepared, &Progress::quiet()).await.unwrap();

        let trace: BacktestTrace =
            serde_json::from_str(&std::fs::read_to_string(&trace_path).unwrap()).unwrap();
        assert_eq!(trace.summary.trade_count, result.trade_count);
        assert_eq!(trace.summary.final_value, result.final_value);
        assert_eq!(trace.summary.starting_cash, DEFAULT_STARTING_CASH);
        assert_eq!(trace.events.len(), 2);
        let first = &trace.events[0];
        assert_eq!(first.symbol, "VTI");
        assert_eq!(first.tactic.as_deref(), Some("always"));
        assert_eq!(first.price, Some(100.0));
        assert_eq!(first.quantity, Some(100.0));
        assert_eq!(first.cash_after, 90_000.0);
        assert_eq!(first.equity_after, 100_000.0);
        // One snapshot for the single simulated day, plus the end.
        assert_eq!(trace.equity.len(), 2);
        assert_eq!(trace.equity.last().unwrap().time, end);
        std::fs::remove_file(&db_path).ok();
        std::fs::remove_file(&config_path).ok();
        std::fs::remove_file(&trace_path).ok();
    }

    #[tokio::test]
    async fn an_unknown_trace_extension_fails_before_any_data_work() {
        let mut opts = options("config_single_tactic.toml", None);
        opts.trace = Some(PathBuf::from("out.txt"));

        assert!(prepare(opts, now(), no_platform, &Progress::quiet())
            .await
            .is_err());
    }

    async fn three_month_run(verbose: bool) -> Vec<crate::progress::Event> {
        let db_path = temp_db();
        let config_path = always_buy_config();
        let first = Utc.with_ymd_and_hms(2026, 1, 5, 14, 30, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 3, 5, 14, 35, 0).unwrap();
        let bars: Vec<Bar> = [(1, 5), (2, 5), (3, 5)]
            .iter()
            .flat_map(|&(month, day)| {
                (0..2).map(move |minute| {
                    let timestamp = Utc
                        .with_ymd_and_hms(2026, month, day, 14, 30 + minute, 0)
                        .unwrap();
                    Bar {
                        timestamp,
                        open: 100.0,
                        high: 100.0,
                        low: 100.0,
                        close: 100.0,
                        volume: 1,
                    }
                })
            })
            .collect();
        let mut connection = db::open_for_write(&db_path).unwrap();
        stock_bars::insert_chunk(
            &mut connection,
            &AssetSymbol::new("VTI"),
            &TimeFrame::OneMinute,
            &((first - Duration::days(35))..end),
            &bars,
        )
        .unwrap();
        drop(connection);
        let mut opts = simulation_options(config_path.clone(), db_path.clone());
        opts.time_range = TimeRangeSpec {
            from: Some("2026-01-05 14:30".to_string()),
            to: Some("2026-03-05 14:35".to_string()),
            ..Default::default()
        };
        let (progress, recorder) = Progress::recording(verbose);

        let prepared = prepare(opts, now(), no_platform, &progress).await.unwrap();
        simulate(prepared, &progress).await.unwrap();

        std::fs::remove_file(&db_path).ok();
        std::fs::remove_file(&config_path).ok();
        recorder.events()
    }

    #[tokio::test]
    async fn one_month_event_is_reported_per_simulated_month() {
        use crate::progress::{Event, MonthDetail};

        let events = three_month_run(false).await;

        let months: Vec<_> = events
            .iter()
            .filter_map(|e| match e {
                Event::Month(m) => Some(m),
                _ => None,
            })
            .collect();
        assert_eq!(months.len(), 3, "{events:?}");
        let labels: Vec<String> = months
            .iter()
            .map(|m| m.month.format("%Y-%m").to_string())
            .collect();
        assert_eq!(labels, vec!["2026-01", "2026-02", "2026-03"]);
        assert!(months
            .iter()
            .all(|m| matches!(m.detail, MonthDetail::Simulated { .. })));
        // The always-buy rule trades once, in January.
        assert!(matches!(
            months[0].detail,
            MonthDetail::Simulated { trades: 1, .. }
        ));
    }

    #[tokio::test]
    async fn phases_are_announced_in_order() {
        use crate::progress::{Event, Phase};

        let events = three_month_run(false).await;

        let phases: Vec<_> = events
            .iter()
            .filter_map(|e| match e {
                Event::Phase(p) => Some(p.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(
            phases,
            vec![
                Phase::CheckingCoverage,
                Phase::LoadingData,
                Phase::Simulating
            ]
        );
    }

    #[tokio::test]
    async fn tick_events_appear_only_when_verbose() {
        use crate::progress::Event;
        let is_tick = |e: &Event| matches!(e, Event::Tick(_));

        let quiet = three_month_run(false).await;
        let loud = three_month_run(true).await;

        assert!(!quiet.iter().any(is_tick));
        assert!(loud.iter().filter(|e| is_tick(e)).count() >= 6);
    }

    #[tokio::test]
    async fn a_zero_interval_is_rejected_before_any_work() {
        let path = std::env::temp_dir().join(format!("greed_cfg_{}.toml", Uuid::new_v4()));
        std::fs::write(&path, "interval = 0\nplatform = \"alpaca\"\n").unwrap();

        let result = prepare(
            simulation_options(path.clone(), std::env::temp_dir().join("unused.duckdb")),
            now(),
            no_platform,
            &Progress::quiet(),
        )
        .await;

        assert!(result.is_err());
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn result_display_shows_the_final_value() {
        let range = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap()
            ..Utc.with_ymd_and_hms(2026, 2, 1, 0, 0, 0).unwrap();

        let text = BacktestResult::new(range, 100_000.0, 105_000.0, 3).to_string();

        assert!(text.contains("trades:        3"), "{text}");
        assert!(text.contains("+5.00%"), "{text}");
        assert!(text.ends_with("Final value: $105000.00"), "{text}");
    }
}
