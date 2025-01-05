mod config_provider;

use crate::error::GreedError;
use crate::strategy::runner::StrategyRunner;

trait StrategyRunnerProvider {
    async fn provide_strategy_runner(&self) -> Result<StrategyRunner, GreedError>;
}