use crate::config::strategy::StrategyConfig;
use crate::error::GreedError;
use crate::platform::FinancialPlatform;
use crate::strategy::provider::StrategyRunnerProvider;
use crate::strategy::reader::read_tactics_from_config;
use crate::strategy::runner::StrategyRunner;
use crate::tactic::TacticRunner;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;

pub struct ConfigStrategyProvider {
    config_path: PathBuf,
    loop_interval: Duration,
    platform: Arc<dyn FinancialPlatform>,
    strategy_config: StrategyConfig,
}

impl ConfigStrategyProvider {
    fn new(
        config_path: PathBuf,
        loop_interval: Duration,
        platform: Arc<dyn FinancialPlatform>,
        strategy_config: StrategyConfig,
    ) -> Self {
        Self {
            config_path,
            loop_interval,
            platform,
            strategy_config,
        }
    }
}

#[async_trait]
impl StrategyRunnerProvider for ConfigStrategyProvider {
    async fn provide_strategy_runner(&self) -> Result<StrategyRunner, GreedError> {
        let tactics = read_tactics_from_config(&self.config_path, &self.strategy_config).await?;
        let tactic_runners = tactics
            .iter()
            .map(|tactic_config| TacticRunner::new(tactic_config.clone(), self.platform.clone()))
            .collect();
        let strategy_runner = StrategyRunner::new(
            self.loop_interval,
            self.strategy_config.properties().clone(),
            tactic_runners,
        );
        Ok(strategy_runner)
    }
}
