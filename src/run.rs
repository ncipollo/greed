use crate::asset::AssetSymbol;
use crate::config::simple::SimpleConfig;
use crate::config::Config;
use crate::db;
use crate::error::GreedError;
use crate::platform;
use crate::platform::FinancialPlatform;
use crate::strategy::factory::StrategyProviderFactory;
use crate::strategy::provider::StrategyRunnerProvider;
use crate::strategy::runner::{StrategyRunner, TacticStrategyRunner};
use duckdb::Connection;
use log::warn;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Default, PartialEq)]
pub struct GreedRunnerArgs {
    pub config_path: PathBuf,
    pub is_simulated: bool,
}

pub struct GreedRunner {
    run_interval: u64,
    config_strategy: Box<dyn StrategyRunner>,
    strategy_providers: Vec<Box<dyn StrategyRunnerProvider>>,
    config_assets: Vec<AssetSymbol>,
    platform: Arc<dyn FinancialPlatform>,
    connection: Arc<Mutex<Connection>>,
}

impl GreedRunner {
    pub async fn new(
        config: Config,
        config_path: PathBuf,
        platform: Arc<dyn FinancialPlatform>,
        connection: Connection,
    ) -> Result<Self, GreedError> {
        let connection = Arc::new(Mutex::new(connection));
        let config_strategy = Box::new(TacticStrategyRunner::from_config(
            &config,
            &platform,
            &connection,
        ));
        let factory = StrategyProviderFactory::new(&config, config_path, &platform, &connection);
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
            connection,
        })
    }

    pub async fn from_args(
        args: GreedRunnerArgs,
        connection: Connection,
    ) -> Result<GreedRunner, GreedError> {
        let config_path = args.config_path.clone();
        let config = GreedRunner::read_config(&args).await?;
        let platform = platform::for_type(&config.platform, args.into())?;
        Self::new(config, config_path, platform, connection).await
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
            sleep(loop_interval).await;
        }
    }

    async fn run_tick(&self, loop_interval: Duration) {
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
            sleep(loop_interval).await;
        }
        // Run the general strategy from the top level config
        self.config_strategy.run(&self.config_assets).await;
        // Capture portfolio status history for this cycle.
        self.record_portfolio_status().await;
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
    // The insert is synchronous for now; switch to tokio::task::spawn_blocking if it proves slow.
    async fn record_portfolio_status(&self) {
        match self.platform.account().await {
            Ok(account) => {
                let connection = self
                    .connection
                    .lock()
                    .expect("portfolio status connection lock");
                if let Err(e) = db::portfolio_status::insert(&connection, &account) {
                    warn!("failed to write portfolio status: {}", e);
                }
            }
            Err(e) => warn!("failed to fetch account for portfolio status: {}", e),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::asset::AssetSymbol;
    use crate::config::Config;
    use crate::db;
    use crate::fixture;
    use crate::platform::account::Account;
    use crate::platform::quote::Quote;
    use crate::platform::MockPlatform;
    use crate::run::GreedRunner;
    use duckdb::Connection;
    use std::path::{Path, PathBuf};
    use std::time::Duration;
    use uuid::Uuid;

    #[tokio::test]
    async fn run_tick_writes_portfolio_status_row_per_tick() {
        let db_path = temp_db_path();
        let connection = db::bootstrap_at(&db_path).expect("bootstrap should succeed");
        let platform = MockPlatform::new().with_account(Account::fixture()).arc();
        let config = Config {
            interval: 1,
            ..Default::default()
        };
        let runner = GreedRunner::new(config, PathBuf::from("unused.toml"), platform, connection)
            .await
            .expect("runner should build");

        for _ in 0..10 {
            runner.run_tick(Duration::from_secs(1)).await;
        }
        drop(runner);

        assert_eq!(portfolio_status_count(&db_path), 10);
        std::fs::remove_file(&db_path).ok();
    }

    #[tokio::test]
    async fn run_tick_writes_stock_quote_row_per_tick() {
        let db_path = temp_db_path();
        let connection = db::bootstrap_at(&db_path).expect("bootstrap should succeed");
        let mut config = fixture::config("config_single_tactic.toml").await;
        config.interval = 0;
        let platform = MockPlatform::new()
            .with_account(Account::fixture())
            .with_quotes(vec![Quote::fixture(AssetSymbol::new("VTI"))])
            .arc();
        let runner = GreedRunner::new(config, PathBuf::from("unused.toml"), platform, connection)
            .await
            .expect("runner should build");

        for _ in 0..10 {
            runner.run_tick(Duration::from_secs(0)).await;
        }
        drop(runner);

        assert_eq!(portfolio_status_count(&db_path), 10);
        assert_eq!(stock_quotes_count(&db_path), 10);
        std::fs::remove_file(&db_path).ok();
    }

    fn temp_db_path() -> PathBuf {
        std::env::temp_dir().join(format!("greed_test_{}.duckdb", Uuid::new_v4()))
    }

    fn portfolio_status_count(db_path: &Path) -> i64 {
        table_row_count(db_path, "portfolio_status")
    }

    fn stock_quotes_count(db_path: &Path) -> i64 {
        table_row_count(db_path, "stock_quotes")
    }

    fn table_row_count(db_path: &Path, table: &str) -> i64 {
        let connection = Connection::open(db_path).expect("reopen connection");
        connection
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .expect("table should be queryable")
    }
}
