use crate::asset::AssetSymbol;
use crate::backtest::BacktestPlatform;
use crate::clock::Timing;
use crate::config::platform::PlatformType;
use crate::config::simple::SimpleConfig;
use crate::config::Config;
use crate::db;
use crate::error::GreedError;
use crate::platform;
use crate::platform::FinancialPlatform;
use crate::quote_accumulator::QuoteAccumulator;
use crate::strategy::factory::StrategyProviderFactory;
use crate::strategy::provider::StrategyRunnerProvider;
use crate::strategy::runner::{StrategyRunner, TacticStrategyRunner};
use log::warn;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Default, PartialEq)]
pub struct GreedRunnerArgs {
    pub config_path: PathBuf,
    pub db_path: Option<PathBuf>,
    pub is_simulated: bool,
    pub record_history: bool,
}

pub struct GreedRunner {
    run_interval: u64,
    config_strategy: Box<dyn StrategyRunner>,
    strategy_providers: Vec<Box<dyn StrategyRunnerProvider>>,
    config_assets: Vec<AssetSymbol>,
    platform: Arc<dyn FinancialPlatform>,
    quote_accumulator: QuoteAccumulator,
    db_path: PathBuf,
    record_history: bool,
    timing: Timing,
}

impl GreedRunner {
    pub async fn new(
        config: Config,
        config_path: PathBuf,
        platform: Arc<dyn FinancialPlatform>,
        db_path: PathBuf,
        record_history: bool,
    ) -> Result<Self, GreedError> {
        Self::with_timing(
            config,
            config_path,
            platform,
            db_path,
            record_history,
            Timing::default(),
        )
        .await
    }

    /// Like `new`, but reads time from `timing` and paces with it. A backtest passes a
    /// simulated clock and no pacing, then drives ticks with `evaluate_strategies`.
    pub async fn with_timing(
        config: Config,
        config_path: PathBuf,
        platform: Arc<dyn FinancialPlatform>,
        db_path: PathBuf,
        record_history: bool,
        timing: Timing,
    ) -> Result<Self, GreedError> {
        let quote_accumulator = QuoteAccumulator::new();
        let config_strategy = Box::new(TacticStrategyRunner::from_config_with_timing(
            &config,
            &platform,
            &quote_accumulator,
            &timing,
        ));
        let factory =
            StrategyProviderFactory::new(&config, config_path, &platform, &quote_accumulator)
                .with_timing(timing.clone());
        let strategy_providers = factory.create_providers().await?;

        // Accumulate assets from all providers
        let mut config_assets = Vec::new();
        for provider in &strategy_providers {
            config_assets.extend(provider.config_assets());
        }
        // Add assets from config strategy
        config_assets.extend(config.tactics.iter().flat_map(|t| t.assets()));

        Ok(Self {
            run_interval: config.interval,
            config_strategy,
            strategy_providers,
            config_assets,
            platform,
            quote_accumulator,
            db_path,
            record_history,
            timing,
        })
    }

    /// A runner for simulations: it can only talk to the given backtest platform, and it has no
    /// database or history recording at all, so a simulation can't write anything down.
    pub async fn for_simulation(
        config: Config,
        config_path: PathBuf,
        platform: Arc<BacktestPlatform>,
        timing: Timing,
    ) -> Result<Self, GreedError> {
        Self::with_timing(config, config_path, platform, PathBuf::new(), false, timing).await
    }

    /// Throws away the quotes gathered this tick. A live run drains them into the database each
    /// tick; a simulation has nowhere to put them and must not let them pile up.
    pub fn discard_pending_quotes(&self) {
        self.quote_accumulator.drain();
    }

    pub async fn from_args(
        args: GreedRunnerArgs,
        db_path: PathBuf,
    ) -> Result<GreedRunner, GreedError> {
        let (runner, _, _) = Self::from_args_with_platform(args, db_path).await?;
        Ok(runner)
    }

    /// Like `from_args`, but also returns the platform type and the live
    /// platform instance it built, so callers (e.g. the `server` command) can
    /// reuse them elsewhere (e.g. in `AppState`) without reconstructing them.
    pub(crate) async fn from_args_with_platform(
        args: GreedRunnerArgs,
        db_path: PathBuf,
    ) -> Result<(GreedRunner, PlatformType, Arc<dyn FinancialPlatform>), GreedError> {
        let config_path = args.config_path.clone();
        let record_history = args.record_history;
        let config = GreedRunner::read_config(&args).await?;
        let platform_type = config.platform.clone();
        let platform = platform::for_type(&platform_type, args.into())?;
        let runner = Self::new(
            config,
            config_path,
            platform.clone(),
            db_path,
            record_history,
        )
        .await?;
        Ok((runner, platform_type, platform))
    }

    async fn read_config(args: &GreedRunnerArgs) -> Result<Config, GreedError> {
        let ext = args.config_path.extension();
        if Some(OsStr::new("csv")) == ext {
            let simple_config = SimpleConfig::from_path(&args.config_path).await?;
            Ok(simple_config.into())
        } else {
            Config::from_path(&args.config_path).await
        }
    }

    pub async fn run_loop(&self) {
        let loop_interval = Duration::from_secs(self.run_interval);
        loop {
            self.run_tick(loop_interval).await;
            self.timing.pacer.pause(loop_interval).await;
        }
    }

    async fn run_tick(&self, loop_interval: Duration) {
        self.evaluate_strategies(loop_interval).await;
        // Capture portfolio status and quote history for this cycle.
        self.record_history().await;
    }

    /// Runs every strategy once against the platform, without recording history. This is the
    /// single step an external loop (such as a backtest) drives.
    pub async fn evaluate_strategies(&self, loop_interval: Duration) {
        // Create strategy runners from providers
        let strategy_runners = match self.provide_strategy_runners().await {
            Ok(runners) => runners,
            Err(e) => {
                // If we fail we will sleep for a bit then try again
                warn!("error creating strategy runners: {}", e);
                return;
            }
        };
        // Run each strategy runner
        for runner in strategy_runners {
            let _ = runner.run(&self.config_assets).await;
            self.timing.pacer.pause(loop_interval).await;
        }
        // Run the general strategy from the top level config
        self.config_strategy.run(&self.config_assets).await;
    }

    async fn provide_strategy_runners(&self) -> Result<Vec<Box<dyn StrategyRunner>>, GreedError> {
        let mut runners = Vec::new();
        for provider in &self.strategy_providers {
            let runner = provider.provide_strategy_runner().await?;
            runners.push(runner);
        }
        Ok(runners)
    }

    // A DB hiccup must not stop trading automation, so failures are logged and swallowed here.
    // Everything that needs the network happens before the DB is opened, so the write lock is
    // held only for the brief inserts. The accumulator is drained unconditionally so it never
    // grows unbounded across ticks, even when `record_history` is disabled.
    // The inserts are synchronous for now; switch to tokio::task::spawn_blocking if they prove slow.
    async fn record_history(&self) {
        let quotes = self.quote_accumulator.drain();
        if !self.record_history {
            return;
        }
        let account = match self.platform.account().await {
            Ok(account) => account,
            Err(e) => {
                warn!("failed to fetch account for portfolio status: {}", e);
                return;
            }
        };
        let mut connection = match db::open_for_write_with_retry(&self.db_path).await {
            Ok(connection) => connection,
            Err(e) => {
                warn!("failed to open db for this tick's writes: {}", e);
                return;
            }
        };
        if let Err(e) = db::portfolio_status::insert(&connection, &account) {
            warn!("failed to write portfolio status: {}", e);
        }
        if !quotes.is_empty() {
            if let Err(e) = db::stock_quotes::insert_many(&mut connection, &quotes) {
                warn!("failed to write stock quotes: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::asset::AssetSymbol;
    use crate::config::Config;
    use crate::db::bootstrap_at;
    use crate::fixture;
    use crate::platform::account::Account;
    use crate::platform::quote::Quote;
    use crate::platform::MockPlatform;
    use crate::run::GreedRunner;
    use duckdb::Connection;
    use std::path::PathBuf;
    use std::time::Duration;
    use uuid::Uuid;

    #[tokio::test]
    async fn run_tick_writes_portfolio_status_row_per_tick() {
        let db_path = temp_db_path();
        bootstrap_at(&db_path).expect("bootstrap should succeed");
        let platform = MockPlatform::new().with_account(Account::fixture()).arc();
        let config = Config {
            interval: 1,
            ..Default::default()
        };
        let runner = GreedRunner::new(
            config,
            PathBuf::from("unused.toml"),
            platform,
            db_path.clone(),
            true,
        )
        .await
        .expect("runner should build");

        for _ in 0..10 {
            runner.run_tick(Duration::from_secs(1)).await;
        }

        assert_eq!(table_row_count(&db_path, "portfolio_status"), 10);
        std::fs::remove_file(&db_path).ok();
    }

    #[tokio::test]
    async fn run_tick_writes_stock_quote_row_per_tick() {
        let db_path = temp_db_path();
        bootstrap_at(&db_path).expect("bootstrap should succeed");
        let mut config = fixture::config("config_single_tactic.toml").await;
        config.interval = 0;
        let platform = MockPlatform::new()
            .with_account(Account::fixture())
            .with_quotes(vec![Quote::fixture(AssetSymbol::new("VTI"))])
            .arc();
        let runner = GreedRunner::new(
            config,
            PathBuf::from("unused.toml"),
            platform,
            db_path.clone(),
            true,
        )
        .await
        .expect("runner should build");

        for _ in 0..10 {
            runner.run_tick(Duration::from_secs(0)).await;
        }

        assert_eq!(table_row_count(&db_path, "portfolio_status"), 10);
        assert_eq!(table_row_count(&db_path, "stock_quotes"), 10);
        std::fs::remove_file(&db_path).ok();
    }

    #[tokio::test]
    async fn run_tick_dedupes_quotes_from_multiple_tactics_watching_same_symbol() {
        let db_path = temp_db_path();
        bootstrap_at(&db_path).expect("bootstrap should succeed");
        let mut config = fixture::config("config_multi_tactic.toml").await;
        config.interval = 0;
        // MockPlatform::latest_quotes ignores the requested symbols, so both of
        // this config's tactics fetch (and would, pre-fix, each write) this same quote.
        let platform = MockPlatform::new()
            .with_account(Account::fixture())
            .with_quotes(vec![Quote::fixture(AssetSymbol::new("VTI"))])
            .arc();
        let runner = GreedRunner::new(
            config,
            PathBuf::from("unused.toml"),
            platform,
            db_path.clone(),
            true,
        )
        .await
        .expect("runner should build");

        for _ in 0..10 {
            runner.run_tick(Duration::from_secs(0)).await;
        }

        assert_eq!(table_row_count(&db_path, "stock_quotes"), 10);
        std::fs::remove_file(&db_path).ok();
    }

    #[tokio::test]
    async fn run_tick_skips_writes_when_record_history_disabled() {
        let db_path = temp_db_path();
        bootstrap_at(&db_path).expect("bootstrap should succeed");
        let mut config = fixture::config("config_strategic.toml").await;
        config.interval = 0;
        let platform = MockPlatform::new()
            .with_account(Account::fixture())
            .with_quotes(vec![Quote::fixture(AssetSymbol::new("VTI"))])
            .arc();
        let runner = GreedRunner::new(
            config,
            fixture::path("config_strategic.toml"),
            platform,
            db_path.clone(),
            false,
        )
        .await
        .expect("runner should build");

        for _ in 0..10 {
            runner.run_tick(Duration::from_secs(0)).await;
        }

        assert_eq!(table_row_count(&db_path, "portfolio_status"), 0);
        assert_eq!(table_row_count(&db_path, "stock_quotes"), 0);
        std::fs::remove_file(&db_path).ok();
    }

    #[tokio::test(start_paused = true)]
    async fn evaluate_strategies_skips_history_and_pacing_when_simulated() {
        use crate::clock::{SimulatedClock, Timing};
        use chrono::{TimeZone, Utc};
        let db_path = temp_db_path();
        bootstrap_at(&db_path).expect("bootstrap should succeed");
        let mut config = fixture::config("config_single_tactic.toml").await;
        config.interval = 3600;
        let platform = MockPlatform::new()
            .with_account(Account::fixture())
            .with_quotes(vec![Quote::fixture(AssetSymbol::new("VTI"))])
            .arc();
        let clock = std::sync::Arc::new(SimulatedClock::new(
            Utc.with_ymd_and_hms(2026, 1, 5, 14, 30, 0).unwrap(),
        ));
        let runner = GreedRunner::with_timing(
            config,
            PathBuf::from("unused.toml"),
            platform,
            db_path.clone(),
            true,
            Timing::simulated(clock),
        )
        .await
        .expect("runner should build");
        let start = tokio::time::Instant::now();

        runner.evaluate_strategies(Duration::from_secs(3600)).await;

        assert_eq!(start.elapsed(), Duration::ZERO);
        assert_eq!(table_row_count(&db_path, "portfolio_status"), 0);
        assert_eq!(table_row_count(&db_path, "stock_quotes"), 0);
        std::fs::remove_file(&db_path).ok();
    }

    fn table_row_count(db_path: &PathBuf, table: &str) -> i64 {
        let connection = Connection::open(db_path).expect("db should be queryable");
        connection
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .expect("table should be queryable")
    }

    fn temp_db_path() -> PathBuf {
        std::env::temp_dir().join(format!("greed_test_{}.duckdb", Uuid::new_v4()))
    }
}
