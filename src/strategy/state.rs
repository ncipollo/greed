use crate::analysis::result::BarsResult;
use crate::asset::AssetSymbol;
use crate::platform::account::Account;
use crate::platform::order::Order;
use crate::platform::position::Position;
use crate::platform::quote::Quote;
use num_decimal::Num;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Default, Eq, PartialEq)]
pub struct StrategyState {
    pub account: Account,
    pub bar_analysis: Rc<HashMap<AssetSymbol, BarsResult>>,
    pub open_orders: HashMap<AssetSymbol, Vec<Order>>,
    pub positions: HashMap<AssetSymbol, Position>,
    pub quotes: HashMap<AssetSymbol, Quote>,
}

impl StrategyState {
    pub fn new(
        account: Account,
        bar_analysis: Rc<HashMap<AssetSymbol, BarsResult>>,
        open_orders: HashMap<AssetSymbol, Vec<Order>>,
        positions: HashMap<AssetSymbol, Position>,
        quotes: HashMap<AssetSymbol, Quote>,
    ) -> Self {
        Self {
            account,
            bar_analysis: bar_analysis.clone(),
            open_orders,
            positions,
            quotes,
        }
    }

    pub fn open_order_value(&self, symbol: &AssetSymbol) -> Num {
        let ask_price = self
            .quotes
            .get(&symbol)
            .map_or(Num::from(0), |quote| quote.ask_price.clone());
        self.open_orders
            .get(&symbol)
            .map_or(Num::from(0), |orders| {
                orders
                    .iter()
                    .map(|order| order.estimated_value(ask_price.clone()))
                    .reduce(|a, b| a + b)
                    .unwrap_or(Num::from(0))
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
        Self {
            account: Account::fixture(),
            bar_analysis: Rc::new(bar_analysis),
            open_orders,
            positions,
            quotes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_order_value() {
        let state = StrategyState::fixture();
        let open_order_value = state.open_order_value(&AssetSymbol::new("SPY"));
        assert_eq!(open_order_value, Num::from(200))
    }

    #[test]
    fn open_order_value_no_orders() {
        let state = StrategyState {
            open_orders: HashMap::new(),
            ..StrategyState::fixture()
        };
        let open_order_value = state.open_order_value(&AssetSymbol::new("SPY"));
        assert_eq!(open_order_value, Num::from(0))
    }

    #[test]
    fn open_order_value_no_quotes() {
        let state = StrategyState {
            open_orders: HashMap::new(),
            ..StrategyState::fixture()
        };
        let open_order_value = state.open_order_value(&AssetSymbol::new("SPY"));
        assert_eq!(open_order_value, Num::from(0))
    }
}