use crate::analysis::AssetAnalyzer;
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
use crate::strategy::state::StrategyState;
use itertools::Itertools;
use log::info;
use num_decimal::Num;
use std::collections::HashMap;
use std::sync::Arc;

mod state;

pub struct StrategyRunner {
    asset_analyzer: AssetAnalyzer,
    config: StrategyConfig,
    platform: Arc<dyn FinancialPlatform>,
}

impl StrategyRunner {
    pub fn new(config: StrategyConfig, platform: Arc<dyn FinancialPlatform>) -> Self {
        Self {
            asset_analyzer: AssetAnalyzer::new(platform.clone()),
            config,
            platform,
        }
    }
    pub async fn run(&self) -> Result<(), GreedError> {
        info!("🧠 running strategy: {}", self.config.name);
        let account = self.fetch_account().await?;
        let symbols = self.config.assets();
        let bar_analysis = self.asset_analyzer.analyze_bars(&symbols).await?;
        let quotes = self.fetch_quotes(&symbols).await?;
        let positions = self.fetch_positions().await?;
        let open_orders = self.fetch_open_orders().await?;

        let _ = StrategyState::new(account, bar_analysis, open_orders, positions, quotes);
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

    async fn fetch_quotes(
        &self,
        symbols: &Vec<AssetSymbol>,
    ) -> Result<HashMap<AssetSymbol, Quote>, GreedError> {
        let symbols_string = Self::symbols_string(&symbols);
        info!("- fetching quotes for {}", symbols_string);
        let quotes = self.platform.latest_quotes(&symbols).await?;
        for quote in &quotes {
            info!("-- {}", quote);
        }
        let by_symbol = quotes
            .into_iter()
            .map(|q| (q.symbol.clone(), q))
            .collect::<HashMap<_, _>>();
        Ok(by_symbol)
    }

    async fn fetch_positions(&self) -> Result<HashMap<AssetSymbol, Position>, GreedError> {
        info!("- fetching open positions");
        let positions = self.platform.positions().await?;
        for position in &positions {
            info!("-- {}", position);
        }
        let by_symbol = positions
            .into_iter()
            .map(|p| (p.symbol.clone(), p))
            .collect::<HashMap<_, _>>();
        Ok(by_symbol)
    }

    async fn fetch_open_orders(&self) -> Result<HashMap<AssetSymbol, Order>, GreedError> {
        info!("- fetching open orders");
        let orders = self.platform.open_orders().await?;
        for order in &orders {
            info!("-- {}", order);
        }
        let by_symbol = orders
            .into_iter()
            .map(|o| (o.symbol.clone(), o))
            .collect::<HashMap<_, _>>();
        Ok(by_symbol)
    }

    fn symbols_string(symbols: &Vec<AssetSymbol>) -> String {
        symbols.iter().map(|s| s.symbol.clone()).join(",")
    }
}
