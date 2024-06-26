use crate::analysis::AssetAnalyzer;
use crate::asset::AssetSymbol;
use crate::config::strategy::StrategyConfig;
use crate::error::GreedError;
use crate::platform::account::Account;
use crate::platform::order::Order;
use crate::platform::position::Position;
use crate::platform::quote::Quote;
use crate::platform::FinancialPlatform;
use crate::strategy::r#do::DoResult;
use crate::strategy::rule::RuleType::{Buy, Sell};
use crate::strategy::rule::{RuleType, StrategyRuleset};
use crate::strategy::state::StrategyState;
use itertools::Itertools;
use log::{info, warn};
use std::collections::HashMap;
use std::sync::Arc;

mod action;
mod r#do;
mod r#for;
mod null;
mod rule;
mod skip;
mod state;
mod target;
mod when;

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

        let state = StrategyState::new(account, bar_analysis, open_orders, positions, quotes);
        self.evaluate_rules(state).await?;
        info!("----------");
        Ok(())
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

    async fn fetch_open_orders(&self) -> Result<HashMap<AssetSymbol, Vec<Order>>, GreedError> {
        info!("- fetching open orders");
        let orders = self.platform.open_orders().await?;
        for order in &orders {
            info!("-- {}", order);
        }
        let by_symbol = orders
            .into_iter()
            .group_by(|o| o.symbol.clone())
            .into_iter()
            .map(|(sym, group)| (sym, group.collect::<Vec<_>>()))
            .collect::<HashMap<_, _>>();
        Ok(by_symbol)
    }

    async fn evaluate_rules(&self, state: StrategyState) -> Result<(), GreedError> {
        let rules = StrategyRuleset::from_config(self.config.clone());
        let buy_result = rules.buy.evaluate(&state);
        let sell_result = rules.sell.evaluate(&state);

        info!("buy rule result: {:?}", buy_result);
        info!("sell rule result: {:?}", sell_result);

        self.perform_resulting_actions(Buy, buy_result).await?;
        self.perform_resulting_actions(Sell, sell_result).await?;
        Ok(())
    }

    async fn perform_resulting_actions(
        &self,
        rule_type: RuleType,
        result: DoResult,
    ) -> Result<(), GreedError> {
        if result.skipped {
            info!(
                "Skipping {rule_type} actions. Reason: {}",
                result.skip_reason
            );
            return Ok(());
        }
        for action in result.actions {
            info!("performing {rule_type} action: {action}");
            let result = self.platform.place_order(action.into_request()).await;

            if let Err(e) = result {
                warn!("error placing order: {e}");
            }
        }
        Ok(())
    }

    fn symbols_string(symbols: &Vec<AssetSymbol>) -> String {
        symbols.iter().map(|s| s.symbol.clone()).join(",")
    }
}
