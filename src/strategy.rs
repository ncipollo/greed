use crate::asset::AssetSymbol;
use crate::config::strategy::StrategyConfig;
use crate::error::GreedError;
use crate::platform::account::Account;
use crate::platform::order::amount::Amount;
use crate::platform::order::Order;
use crate::platform::position::Position;
use crate::platform::quote::Quote;
use crate::platform::request::OrderRequest;
use crate::platform::FinancialPlatform;
use itertools::Itertools;
use log::info;
use num_decimal::Num;
use std::sync::Arc;

mod symbols;

pub struct StrategyRunner {
    config: StrategyConfig,
    platform: Arc<dyn FinancialPlatform>,
}

impl StrategyRunner {
    pub fn new(config: StrategyConfig, platform: Arc<dyn FinancialPlatform>) -> Self {
        Self { config, platform }
    }
    pub async fn run(&self) -> Result<(), GreedError> {
        info!("🧠 running strategy: {}", self.config.name);
        // let _ = self.test_buy().await?;
        let _ = self.fetch_account().await?;
        let symbols = symbols::from_config(&self.config);
        let _ = self.fetch_quotes(symbols).await?;
        let _ = self.fetch_positions().await?;
        let _ = self.fetch_open_orders().await?;
        info!("----------");
        Ok(())
    }

    async fn test_buy(&self) -> Result<Order, GreedError> {
        info!("- test buy of VTI");
        let request = OrderRequest::market_order_buy(
            AssetSymbol::new("VTI"),
            Amount::Notional(Num::from(10)),
        );
        self.platform.place_order(request).await
    }

    async fn fetch_account(&self) -> Result<Account, GreedError> {
        info!("- fetching account info");
        let account = self.platform.account().await?;
        info!("-- {}", account);
        Ok(account)
    }

    async fn fetch_quotes(&self, symbols: Vec<AssetSymbol>) -> Result<Vec<Quote>, GreedError> {
        let symbols_string = Self::symbols_string(&symbols);
        info!("- fetching quotes for {}", symbols_string);
        let quotes = self.platform.latest_quotes(&symbols).await?;
        for quote in &quotes {
            info!("-- {}", quote);
        }
        Ok(quotes)
    }

    async fn fetch_positions(&self) -> Result<Vec<Position>, GreedError> {
        info!("- fetching open positions");
        let positions = self.platform.positions().await?;
        for position in &positions {
            info!("-- {}", position);
        }
        Ok(positions)
    }

    async fn fetch_open_orders(&self) -> Result<Vec<Order>, GreedError> {
        info!("- fetching open orders");
        let orders = self.platform.open_orders().await?;
        for order in &orders {
            info!("-- {}", order);
        }
        Ok(orders)
    }

    fn symbols_string(symbols: &Vec<AssetSymbol>) -> String {
        symbols.iter().map(|s| s.symbol.clone()).join(",")
    }
}
