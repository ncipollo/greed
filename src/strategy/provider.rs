pub mod config_provider;

use async_trait::async_trait;
use crate::error::GreedError;
use crate::strategy::runner::StrategyRunner;
use crate::asset::AssetSymbol;

#[async_trait]
pub trait StrategyRunnerProvider {
    async fn provide_strategy_runner(&self) -> Result<StrategyRunner, GreedError>;
    fn config_assets(&self) -> Vec<AssetSymbol>;
}