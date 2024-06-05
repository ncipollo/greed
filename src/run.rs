use crate::config::Config;
use crate::error::GreedError;
use crate::platform::FinancialPlatform;
use crate::strategy::StrategyRunner;
use crate::platform;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use log::warn;
use tokio::time::sleep;

#[derive(Debug, Default, PartialEq)]
pub struct GreedRunnerArgs {
    pub config_path: PathBuf,
    pub is_simulated: bool,
}

pub struct GreedRunner {
    run_interval: u64,
    strategy_runners: Vec<StrategyRunner>,
}

impl GreedRunner {
    pub fn new(config: Config, platform: Arc<dyn FinancialPlatform>) -> Self {
        let strategy_runners = config
            .strategies
            .into_iter()
            .map(|c| StrategyRunner::new(c.clone(), platform.clone()))
            .collect::<Vec<_>>();
        Self {
            run_interval: config.interval,
            strategy_runners,
        }
    }

    pub async fn from_args(args: GreedRunnerArgs) -> Result<GreedRunner, GreedError> {
        let config = Config::from_path(&args.config_path).await?;
        let platform = platform::for_type(&config.platform, args.into())?;
        Ok(Self::new(config, platform))
    }

    pub async fn run_loop(&self) -> Result<(), GreedError> {
        let loop_interval = Duration::from_secs(self.run_interval);
        let mut strategy_index = 0;
        loop {
            let _ = self.strategy_runners[strategy_index].run().await
                .inspect_err(|e| warn!("{e}"));
            strategy_index = (strategy_index + 1) % self.strategy_runners.len();
            sleep(loop_interval).await;
        }
    }
}
