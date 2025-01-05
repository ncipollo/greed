use std::time::Duration;
use log::warn;
use tokio::time::sleep;
use crate::config::strategy::StrategyProperties;
use crate::error::GreedError;
use crate::tactic::TacticRunner;

pub struct StrategyRunner {
    loop_interval: Duration,
    strategy_properties: StrategyProperties,
    tactic_runners: Vec<TacticRunner>,
}

impl StrategyRunner {
    pub fn new(
        loop_interval: Duration,
        strategy_properties: StrategyProperties,
        tactic_runners: Vec<TacticRunner>,
    ) -> Self {
        Self {
            loop_interval,
            strategy_properties,
            tactic_runners,
        }
    }

    pub async fn run_loop(&self) -> Result<(), GreedError> {
        for tactic_runner in &self.tactic_runners {
            let _ = tactic_runner.run().await.inspect_err(|e| warn!("{e}"));
            sleep(self.loop_interval).await;
        }
        Ok(())
    }
}