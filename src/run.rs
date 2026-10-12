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
use std::sync::Arc;
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
    connection: Connection,
}

impl GreedRunner {
    pub async fn new(
        config: Config,
        config_path: PathBuf,
        platform: Arc<dyn FinancialPlatform>,
        connection: Connection,
    ) -> Result<Self, GreedError> {
        let config_strategy = Box::new(TacticStrategyRunner::from_config(&config, &platform));
        let factory = StrategyProviderFactory::new(&config, config_path, &platform);
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
            // Create strategy runners from providers
            let strategy_runners_result = self.provide_strategy_runners().await;
            if let Err(e) = strategy_runners_result {
                // If we fail we will sleep for a bit then try again
                warn!("error creating strategy runners: {}", e);
                sleep(loop_interval).await;
                continue;
            }
            // Run each strategy runner
            let strategy_runners = strategy_runners_result.unwrap();
            for runner in strategy_runners {
                let _ = runner.run(&self.config_assets).await;
                sleep(loop_interval).await;
            }
            // Run the general strategy from the top level config
            self.config_strategy.run(&self.config_assets).await;
            // Capture portfolio status history for this cycle.
            self.record_portfolio_status().await;
            // Sleep for the loop interval then start again.
            sleep(loop_interval).await;
        }
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
                if let Err(e) = db::portfolio_status::insert(&self.connection, &account) {
                    warn!("failed to write portfolio status: {}", e);
                }
            }
            Err(e) => warn!("failed to fetch account for portfolio status: {}", e),
        }
    }
}
