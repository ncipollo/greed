use crate::analysis::result::BarsResult;
use crate::analysis::AssetAnalyzer;
use crate::asset::AssetSymbol;
use crate::config::quote_fetcher_config::QuoteFetcherConfig;
use crate::config::strategy::StrategyProperties;
use crate::config::tactic::TacticConfig;
use crate::error::GreedError;
use crate::platform::account::Account;
use crate::platform::order::Order;
use crate::platform::position::Position;
use crate::platform::quote::Quote;
use crate::platform::FinancialPlatform;
use crate::tactic::r#do::DoResult;
use crate::tactic::rule::RuleType::{Buy, Sell};
use crate::tactic::rule::{RuleType, TacticRuleset};
use crate::tactic::state::TacticState;
use itertools::Itertools;
use log::{info, warn};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

mod action;
mod filter;
mod r#do;
mod r#for;
mod null;
mod rule;
mod skip;
mod state;
mod target;
mod when;

pub struct TacticRunner {
    asset_analyzer: AssetAnalyzer,
    config: TacticConfig,
    platform: Arc<dyn FinancialPlatform>,
    strategy_properties: StrategyProperties,
}

impl TacticRunner {
    pub fn new(
        config: TacticConfig,
        platform: Arc<dyn FinancialPlatform>,
        strategy_properties: StrategyProperties,
    ) -> Self {
        Self {
            asset_analyzer: AssetAnalyzer::new(platform.clone()),
            config,
            platform,
            strategy_properties,
        }
    }
    pub async fn run(&self, config_assets: &[AssetSymbol]) -> Result<(), GreedError> {
        info!("🧠 running tactic: {}", self.config.name);
        let account = self.fetch_account().await?;
        let symbols = self.config.assets();
        let positions = self.fetch_positions().await?;
        let open_orders = self.fetch_open_orders().await?;

        let (bar_analysis, quotes): (
            Rc<HashMap<AssetSymbol, BarsResult>>,
            HashMap<AssetSymbol, Quote>,
        ) = if self.config.should_fetch_quotes() {
            let bar_analysis = self.asset_analyzer.analyze_bars(&symbols).await?;
            let quotes: HashMap<AssetSymbol, Quote> = self.fetch_quotes(&symbols).await?;
            (bar_analysis, quotes)
        } else {
            (Rc::new(HashMap::new()), HashMap::new())
        };

        let state = TacticState::new(
            account,
            bar_analysis,
            open_orders,
            positions,
            quotes,
            self.strategy_properties.clone(),
            config_assets.to_vec(),
        );
        self.evaluate_rules(state).await?;
        info!("----------");
        Ok(())
    }

    async fn fetch_account(&self) -> Result<Account, GreedError> {
        let account = self.platform.account().await?;
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
        let positions = self.platform.positions().await?;
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

    async fn evaluate_rules(&self, state: TacticState) -> Result<(), GreedError> {
        let rules = TacticRuleset::from_config(self.config.clone());
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
