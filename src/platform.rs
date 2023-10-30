mod alpaca;

use crate::config::platform::PlatformType;
use crate::error::GreedError;
use crate::platform::alpaca::AlpacaPlatform;

pub trait FinancialPlatform {}

pub fn for_type(platform_type: PlatformType) -> Result<Box<dyn FinancialPlatform>, GreedError> {
    let platform = match platform_type {
        PlatformType::Alpaca => Box::new(AlpacaPlatform::new()),
    };
    Ok(platform)
}
