use crate::analysis::result::BarsResult;
use crate::asset::AssetSymbol;
use crate::platform::account::Account;
use crate::platform::order::Order;
use crate::platform::position::Position;
use crate::platform::quote::Quote;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Default, Eq, PartialEq)]
pub struct StrategyState {
    pub account: Account,
    pub bar_analysis: Rc<HashMap<AssetSymbol, BarsResult>>,
    pub open_orders: HashMap<AssetSymbol, Order>,
    pub positions: HashMap<AssetSymbol, Position>,
    pub quotes: HashMap<AssetSymbol, Quote>,
}

impl StrategyState {
    pub fn new(
        account: Account,
        bar_analysis: Rc<HashMap<AssetSymbol, BarsResult>>,
        open_orders: HashMap<AssetSymbol, Order>,
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

    #[cfg(test)]
    pub fn fixture() -> Self {
        let spy = AssetSymbol::new("SPY");
        let vti = AssetSymbol::new("VTI");

        let bar_analysis = HashMap::from([
            (spy.clone(), BarsResult::fixture(spy.clone())),
            (vti.clone(), BarsResult::fixture(vti.clone())),
        ]);
        let open_orders = HashMap::from([
            (spy.clone(), Order::fixture(spy.clone())),
            (vti.clone(), Order::fixture(vti.clone())),
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
            quotes
        }
    }
}
