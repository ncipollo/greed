use crate::config::strategy::StrategyProperties;
use crate::config::Config;
use crate::error::GreedError;
use crate::platform::FinancialPlatform;
use crate::tactic::TacticRunner;
use log::warn;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

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

    pub fn from_config(config: Config, platform: Arc<dyn FinancialPlatform>) -> Self {
        let loop_interval = Duration::from_secs(config.interval);

        let tactic_runners = config
            .tactics
            .into_iter()
            .map(|tactic_config| TacticRunner::new(tactic_config.clone(), platform.clone()))
            .collect();
        Self::new(loop_interval, Default::default(), tactic_runners)
    }

    #[cfg(test)]
    pub fn tactic_runner_count(&self) -> usize {
        self.tactic_runners.len()
    }

    pub async fn run_loop(&self) -> Result<(), GreedError> {
        for tactic_runner in &self.tactic_runners {
            let _ = tactic_runner.run().await.inspect_err(|e| warn!("{e}"));
            sleep(self.loop_interval).await;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture;
    use crate::platform::noop::NoOpPlatform;

    #[tokio::test]
    async fn from_config_empty_tactics() {
        let config = fixture::config("config_minimal.toml").await;
        let runner = StrategyRunner::from_config(config, NoOpPlatform::arc());
        assert_eq!(runner.loop_interval, Duration::from_secs(60));
        assert_eq!(runner.tactic_runner_count(), 0);
    }

    #[tokio::test]
    async fn from_config_with_tactics() {
        let config = fixture::config("config_single_tactic.toml").await;
        let runner = StrategyRunner::from_config(config, NoOpPlatform::arc());
        assert_eq!(runner.loop_interval, Duration::from_secs(300));
        assert_eq!(runner.tactic_runner_count(), 1);
    }
}
