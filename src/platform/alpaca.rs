mod factory;

use crate::error::GreedError;
use crate::platform::alpaca::factory::create_alpaca_client;
use crate::platform::FinancialPlatform;
use crate::run::GreedRunnerArgs;
use apca::Client;

pub struct AlpacaPlatform {
    client: Client,
}

impl AlpacaPlatform {
    pub fn new(runner_args: &GreedRunnerArgs) -> Result<Self, GreedError> {
        Ok(Self {
            client: create_alpaca_client(runner_args.is_simulated)?,
        })
    }
}

impl FinancialPlatform for AlpacaPlatform {}
