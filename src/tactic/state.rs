use std::collections::HashMap;
use std::rc::Rc;

use crate::analysis::result::BarsResult;
use crate::asset::AssetSymbol;
use crate::config::strategy::StrategyProperties;
use crate::platform::account::Account;
use crate::platform::order::Order;
use crate::platform::position::Position;
use crate::platform::quote::Quote;

#[derive(Default, PartialEq)]
pub struct TacticState {
    pub account: Account,
    pub bar_analysis: Rc<HashMap<AssetSymbol, BarsResult>>,
    pub open_orders: HashMap<AssetSymbol, Vec<Order>>,
    pub positions: HashMap<AssetSymbol, Position>,
    pub quotes: HashMap<AssetSymbol, Quote>,
    pub strategy_properties: StrategyProperties
}

impl TacticState {
    pub fn new(
        account: Account,
        bar_analysis: Rc<HashMap<AssetSymbol, BarsResult>>,
        open_orders: HashMap<AssetSymbol, Vec<Order>>,
        positions: HashMap<AssetSymbol, Position>,
        quotes: HashMap<AssetSymbol, Quote>,
        strategy_properties: StrategyProperties
    ) -> Self {
        Self {
            account,
            bar_analysis: bar_analysis.clone(),
            open_orders,
            positions,
            quotes,
            strategy_properties
        }
    }

    pub fn open_order_value(&self, symbol: &AssetSymbol) -> f64 {
        let ask_price = self
            .quotes
            .get(&symbol)
            .map_or(0.0, |quote| quote.ask_price.clone());
        self.open_orders.get(&symbol).map_or(0.0, |orders| {
            orders
                .iter()
                .map(|order| order.estimated_value(ask_price.clone()))
                .reduce(|a, b| a + b)
                .unwrap_or(0.0)
        })
    }

    #[cfg(test)]
    pub fn fixture() -> Self {
        let spy = AssetSymbol::new("SPY");
        let vti = AssetSymbol::new("VTI");

        let bar_analysis = HashMap::from([
            (spy.clone(), BarsResult::fixture(spy.clone())),
            (vti.clone(), BarsResult::fixture(vti.clone())),
        ]);
        let open_orders = HashMap::from([
            (spy.clone(), vec![Order::fixture(spy.clone())]),
            (vti.clone(), vec![Order::fixture(vti.clone())]),
        ]);
        let positions = HashMap::from([
            (spy.clone(), Position::fixture(spy.clone())),
            (vti.clone(), Position::fixture(vti.clone())),
        ]);
        let quotes = HashMap::from([
            (spy.clone(), Quote::fixture(spy.clone())),
            (vti.clone(), Quote::fixture(vti.clone())),
        ]);
        let strategy_properties = StrategyProperties {
            name: "test".to_string(),
            portfolio_percent: 50.0
        };
        Self {
            account: Account::fixture(),
            bar_analysis: Rc::new(bar_analysis),
            open_orders,
            positions,
            quotes,
            strategy_properties
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_order_value() {
        let state = TacticState::fixture();
        let open_order_value = state.open_order_value(&AssetSymbol::new("SPY"));
        assert_eq!(open_order_value, 200.0)
    }

    #[test]
    fn open_order_value_no_orders() {
        let state = TacticState {
            open_orders: HashMap::new(),
            ..TacticState::fixture()
        };
        let open_order_value = state.open_order_value(&AssetSymbol::new("SPY"));
        assert_eq!(open_order_value, 0.0)
    }

    #[test]
    fn open_order_value_no_quotes() {
        let state = TacticState {
            open_orders: HashMap::new(),
            ..TacticState::fixture()
        };
        let open_order_value = state.open_order_value(&AssetSymbol::new("SPY"));
        assert_eq!(open_order_value, 0.0)
    }
}
