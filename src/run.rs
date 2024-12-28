use crate::config::simple::SimpleConfig;
use crate::config::Config;
use crate::error::GreedError;
use crate::platform;
use crate::platform::FinancialPlatform;
use crate::tactic::TacticRunner;
use log::warn;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Default, PartialEq)]
pub struct GreedRunnerArgs {
    pub config_path: PathBuf,
    pub is_simulated: bool,
}

pub struct GreedRunner {
    run_interval: u64,
    tactic_runners: Vec<TacticRunner>,
}

impl GreedRunner {
    pub fn new(config: Config, platform: Arc<dyn FinancialPlatform>) -> Self {
        let tactic_runners = config
            .tactics
            .into_iter()
            .map(|c| TacticRunner::new(c.clone(), platform.clone()))
            .collect::<Vec<_>>();
        Self {
            run_interval: config.interval,
            tactic_runners,
        }
    }

    pub async fn from_args(args: GreedRunnerArgs) -> Result<GreedRunner, GreedError> {
        let config = GreedRunner::read_config(&args).await?;
        let platform = platform::for_type(&config.platform, args.into())?;
        Ok(Self::new(config, platform))
    }

    async fn read_config(args: &GreedRunnerArgs) -> Result<Config, GreedError> {
        let ext = args.config_path.extension();
        if Some(OsStr::new("csv")) == ext {
            let simple_config = SimpleConfig::from_path(&args.config_path).await?;
            Ok(simple_config.into())
        } else {
            Config::from_path(&args.config_path).await
        }
    }

    pub async fn run_loop(&self) -> Result<(), GreedError> {
        let loop_interval = Duration::from_secs(self.run_interval);
        let mut tactic_index = 0;
        loop {
            let _ = self.tactic_runners[tactic_index]
                .run()
                .await
                .inspect_err(|e| warn!("{e}"));
            tactic_index = (tactic_index + 1) % self.tactic_runners.len();
            sleep(loop_interval).await;
        }
    }
}
