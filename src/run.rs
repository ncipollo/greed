use crate::config::Config;
use crate::error::GreedError;
use crate::platform;
use crate::platform::FinancialPlatform;
use std::path::PathBuf;
use std::time::Duration;
use log::info;
use tokio::time::sleep;

#[derive(Debug, Default, PartialEq)]
pub struct GreedRunnerArgs {
    pub config_path: PathBuf,
    pub is_simulated: bool,
}

pub struct GreedRunner {
    config: Config,
    platform: Box<dyn FinancialPlatform>,
}

impl GreedRunner {
    pub fn new(config: Config, platform: Box<dyn FinancialPlatform>) -> Self {
        Self { config, platform }
    }

    pub async fn from_args(args: GreedRunnerArgs) -> Result<Self, GreedError> {
        let config = Config::from_path(&args.config_path).await?;
        let platform = platform::for_type(&config.platform, &args)?;
        Ok(Self::new(config, platform))
    }

    pub async fn run_loop(&self) {
        loop {
            info!(" 💵");
            sleep(Duration::from_millis(1_000)).await;
        }
    }
}
