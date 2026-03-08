use crate::asset::AssetSymbol;
use crate::config::agent::AgentConfig;
use crate::config::strategy::StrategyConfig;
use crate::error::GreedError;
use crate::strategy::path::path_for_config;
use crate::strategy::provider::StrategyRunnerProvider;
use crate::strategy::runner::{StrategyRunner, TacticStrategyRunner};
use async_trait::async_trait;
use log::info;
use std::path::PathBuf;
use std::time::Duration;

pub struct AgentStrategyProvider {
    strategy_config: StrategyConfig,
    agent_config: AgentConfig,
    loop_interval: Duration,
}

impl AgentStrategyProvider {
    pub async fn new(
        config_path: PathBuf,
        loop_interval: Duration,
        strategy_config: StrategyConfig,
    ) -> Result<Self, GreedError> {
        let agent_path = path_for_config(&config_path, &strategy_config)?;
        let agent_config = AgentConfig::from_path(agent_path).await?;
        Ok(Self {
            strategy_config,
            agent_config,
            loop_interval,
        })
    }
}

#[async_trait]
impl StrategyRunnerProvider for AgentStrategyProvider {
    async fn provide_strategy_runner(&self) -> Result<Box<dyn StrategyRunner>, GreedError> {
        let name = self.strategy_config.properties().name;
        info!("running agent strategy: {name}");
        let runner = TacticStrategyRunner::new(
            self.loop_interval,
            self.strategy_config.properties().clone(),
            vec![],
        );
        Ok(Box::new(runner))
    }

    fn config_assets(&self) -> Vec<AssetSymbol> {
        self.agent_config
            .allow
            .iter()
            .map(|s| AssetSymbol::new(s))
            .collect()
    }
}
