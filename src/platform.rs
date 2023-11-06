pub mod account;
mod alpaca;
pub mod args;
mod id;
pub mod quote;

use crate::asset::AssetSymbol;
use crate::config::platform::PlatformType;
use crate::error::GreedError;
use crate::platform::account::Account;
use crate::platform::alpaca::AlpacaPlatform;
use crate::platform::args::PlatformArgs;
use crate::platform::quote::Quote;
use async_trait::async_trait;
use log::info;

#[async_trait]
pub trait FinancialPlatform {
    async fn account(&self) -> Result<Account, GreedError>;
    async fn latest_quotes(&self, symbols: &Vec<AssetSymbol>) -> Result<Vec<Quote>, GreedError>;
}

pub fn for_type(
    platform_type: &PlatformType,
    args: PlatformArgs,
) -> Result<Box<dyn FinancialPlatform>, GreedError> {
    info!("🏦 Using financial platform - {:?}", platform_type);
    let platform = match platform_type {
        PlatformType::Alpaca => Box::new(AlpacaPlatform::new(&args)?),
    };
    Ok(platform)
}
