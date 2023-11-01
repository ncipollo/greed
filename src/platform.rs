mod alpaca;

use log::info;
use crate::config::platform::PlatformType;
use crate::error::GreedError;
use crate::platform::alpaca::AlpacaPlatform;
use crate::run::GreedRunnerArgs;

pub trait FinancialPlatform {}

pub fn for_type(
    platform_type: &PlatformType,
    args: &GreedRunnerArgs,
) -> Result<Box<dyn FinancialPlatform>, GreedError> {
    info!("🏦 Using financial platform - {:?}", platform_type);
    let platform = match platform_type {
        PlatformType::Alpaca => Box::new(AlpacaPlatform::new(args)?),
    };
    Ok(platform)
}
