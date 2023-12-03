pub mod account;
mod alpaca;
pub mod args;
pub mod asset_class;
pub mod bar;
pub mod bars;
pub mod id;
pub mod order;
pub mod position;
pub mod quote;
pub mod request;
pub mod side;

use crate::asset::AssetSymbol;
use crate::config::platform::PlatformType;
use crate::error::GreedError;
use crate::platform::account::Account;
use crate::platform::alpaca::AlpacaPlatform;
use crate::platform::args::PlatformArgs;
use crate::platform::bar::bar_request::BarRequest;
use crate::platform::bars::Bars;
use crate::platform::order::Order;
use crate::platform::position::Position;
use crate::platform::quote::Quote;
use crate::platform::request::OrderRequest;
use async_trait::async_trait;
use log::info;
use std::sync::Arc;

#[async_trait]
pub trait FinancialPlatform {
    async fn account(&self) -> Result<Account, GreedError>;
    async fn bars(&self, bars_request: BarRequest) -> Result<Bars, GreedError>;
    async fn latest_quotes(&self, symbols: &Vec<AssetSymbol>) -> Result<Vec<Quote>, GreedError>;
    async fn place_order(&self, order_request: OrderRequest) -> Result<Order, GreedError>;
    async fn positions(&self) -> Result<Vec<Position>, GreedError>;
    async fn open_orders(&self) -> Result<Vec<Order>, GreedError>;
}

pub fn for_type(
    platform_type: &PlatformType,
    args: PlatformArgs,
) -> Result<Arc<dyn FinancialPlatform>, GreedError> {
    info!("🏦 Using financial platform - {:?}", platform_type);
    let platform = match platform_type {
        PlatformType::Alpaca => Arc::new(AlpacaPlatform::new(&args)?),
    };
    Ok(platform)
}
