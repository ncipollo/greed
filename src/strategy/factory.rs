use crate::config::strategy::StrategyConfig;
use crate::config::Config;
use crate::error::GreedError;
use crate::platform::FinancialPlatform;
use crate::strategy::provider::config_provider::ConfigStrategyProvider;
use crate::strategy::provider::StrategyRunnerProvider;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

pub struct StrategyProviderFactory<'a> {
    config: &'a Config,
    config_path: PathBuf,
    platform: &'a Arc<dyn FinancialPlatform>,
}

impl<'a> StrategyProviderFactory<'a> {
    pub fn new(
        config: &'a Config,
        config_path: PathBuf,
        platform: &'a Arc<dyn FinancialPlatform>,
    ) -> StrategyProviderFactory<'a> {
        Self {
            config,
            config_path,
            platform,
        }
    }

    pub async fn create_providers(
        &self,
    ) -> Result<Vec<Box<dyn StrategyRunnerProvider>>, GreedError> {
        let strategy_configs = self.config.strategies.clone();
        let mut providers = Vec::new();
        for strategy_config in strategy_configs {
            let provider = self.provider_for_strategy_config(strategy_config).await?;
            providers.push(provider);
        }
        Ok(providers)
    }

    async fn provider_for_strategy_config(
        &self,
        strategy_config: StrategyConfig,
    ) -> Result<Box<dyn StrategyRunnerProvider>, GreedError> {
        match strategy_config {
            StrategyConfig::LocalFile { .. } => {
                let provider = ConfigStrategyProvider::new(
                    self.config_path.clone(),
                    Duration::from_secs(self.config.interval),
                    self.platform.clone(),
                    strategy_config,
                )
                .await?;
                Ok(Box::new(provider))
            }
        }
    }
}
