pub mod agent_provider;
pub mod config_provider;

use crate::asset::AssetSymbol;
use crate::error::GreedError;
use crate::strategy::runner::StrategyRunner;
use async_trait::async_trait;

#[async_trait]
pub trait StrategyRunnerProvider {
    async fn provide_strategy_runner(&self) -> Result<StrategyRunner, GreedError>;
    fn config_assets(&self) -> Vec<AssetSymbol>;
}
