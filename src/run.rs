use crate::config::simple::SimpleConfig;
use crate::config::Config;
use crate::error::GreedError;
use crate::platform;
use crate::platform::FinancialPlatform;
use crate::strategy::factory::StrategyProviderFactory;
use crate::strategy::provider::StrategyRunnerProvider;
use crate::strategy::runner::StrategyRunner;
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
    config_strategy: StrategyRunner,
    strategy_providers: Vec<Box<dyn StrategyRunnerProvider>>,
}

impl GreedRunner {
    pub async fn new(
        config: Config,
        config_path: PathBuf,
        platform: Arc<dyn FinancialPlatform>,
    ) -> Result<Self, GreedError> {
        let config_strategy = StrategyRunner::from_config(&config, &platform);
        let factory = StrategyProviderFactory::new(&config, config_path, &platform);
        let strategy_providers = factory.create_providers().await?;
        Ok(Self {
            run_interval: config.interval,
            config_strategy,
            strategy_providers,
        })
    }

    pub async fn from_args(args: GreedRunnerArgs) -> Result<GreedRunner, GreedError> {
        let config_path = args.config_path.clone();
        let config = GreedRunner::read_config(&args).await?;
        let platform = platform::for_type(&config.platform, args.into())?;
        Ok(Self::new(config, config_path, platform).await?)
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
                let _ = runner.run().await;
                sleep(loop_interval).await;
            }
            // Run the general strategy from the top level config
            self.config_strategy.run().await;
            // Sleep for the loop interval then start again.
            sleep(loop_interval).await;
        }
    }

    async fn provide_strategy_runners(&self) -> Result<Vec<StrategyRunner>, GreedError> {
        let mut runners = Vec::new();
        for provider in &self.strategy_providers {
            let runner = provider.provide_strategy_runner().await?;
            runners.push(runner);
        }
        Ok(runners)
    }
}
