use crate::config::strategy::StrategyConfig;
use crate::config::tactic::TacticConfig;
use crate::error::GreedError;
use crate::platform::FinancialPlatform;
use crate::strategy::provider::StrategyRunnerProvider;
use crate::strategy::reader::read_tactics_from_config;
use crate::strategy::runner::StrategyRunner;
use crate::tactic::TacticRunner;
use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use crate::asset::AssetSymbol;

pub struct ConfigStrategyProvider {
    loop_interval: Duration,
    platform: Arc<dyn FinancialPlatform>,
    strategy_config: StrategyConfig,
    tactic_configs: Vec<TacticConfig>,
}

impl ConfigStrategyProvider {
    pub async fn new(
        config_path: PathBuf,
        loop_interval: Duration,
        platform: Arc<dyn FinancialPlatform>,
        strategy_config: StrategyConfig,
    ) -> Result<Self, GreedError> {
        let tactic_configs = read_tactics_from_config(&config_path, &strategy_config).await?;
        Ok(Self {
            loop_interval,
            platform,
            strategy_config,
            tactic_configs,
        })
    }
}

#[async_trait]
impl StrategyRunnerProvider for ConfigStrategyProvider {
    async fn provide_strategy_runner(&self) -> Result<StrategyRunner, GreedError> {
        let tactic_runners = self
            .tactic_configs
            .iter()
            .map(|tactic_config| {
                TacticRunner::new(
                    tactic_config.clone(),
                    self.platform.clone(),
                    self.strategy_config.properties(),
                )
            })
            .collect();
        let strategy_runner = StrategyRunner::new(
            self.loop_interval,
            self.strategy_config.properties().clone(),
            tactic_runners,
        );
        Ok(strategy_runner)
    }

    fn config_assets(&self) -> Vec<AssetSymbol> {
        self.tactic_configs
            .iter()
            .flat_map(|tactic_config| tactic_config.assets())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture;
    use crate::platform::noop::NoOpPlatform;

    #[tokio::test]
    async fn provide_strategy_runner_no_tactics() {
        let provider = provider_for_config("config_minimal.toml").await;
        let strategy_runner = provider
            .provide_strategy_runner()
            .await
            .expect("should return runner");
        assert_eq!(strategy_runner.tactic_runner_count(), 0);
    }

    #[tokio::test]
    async fn provide_strategy_runner_with_tactics() {
        let provider = provider_for_config("config_multi_tactic.toml").await;
        let strategy_runner = provider
            .provide_strategy_runner()
            .await
            .expect("should return runner");
        assert_ne!(strategy_runner.tactic_runner_count(), 0);
    }

    #[tokio::test]
    async fn config_assets() {
        let provider = provider_for_config("config_multi_tactic.toml").await;
        let assets = provider.config_assets();
        assert_eq!(assets.len(), 2);
        assert!(assets.contains(&AssetSymbol::new("VTI")));
        assert!(assets.contains(&AssetSymbol::new("UVXY")));
    }

    async fn provider_for_config(config_name: &str) -> ConfigStrategyProvider {
        let strategy_config = StrategyConfig::LocalFile {
            path: config_name.to_string(),
            properties: Default::default(),
        };
        ConfigStrategyProvider::new(
            fixture::path(config_name),
            Duration::from_secs(1),
            NoOpPlatform::arc(),
            strategy_config,
        )
        .await
        .expect("should create provider")
    }
}
