use crate::num::NumPercent;
use crate::strategy::r#do::{DoResult, DoRule};
use crate::strategy::state::StrategyState;
use crate::strategy::target::TargetAsset;
use crate::strategy::when::WhenResult;
use num_decimal::Num;

pub struct DoBuyRule {
    buy_percent: f64,
}

impl DoBuyRule {
    pub fn boxed(buy_percent: f64) -> Box<dyn DoRule> {
        Box::new(Self { buy_percent })
    }

    fn calculate_buy_amount(&self, state: &StrategyState, target_asset: &TargetAsset) -> f64 {
        let cash = state.account.cash.clone();
        let equity = state.account.equity.clone();
        let target_percent = self.target_percent(target_asset);
        let desired_amount = equity.percent_of(target_percent);
        // Get position, get orders, and get account
        // total desired amount = equity * target percent
        // buy amount  = total desired position - postion - pending orders
        0.0
    }

    fn target_percent(&self, target_asset: &TargetAsset) -> f64 {
        (target_asset.percent * self.buy_percent) / 100.0
    }

    fn position_amount(&self, state: StrategyState, target_asset: TargetAsset) -> Num {
        let position = state.positions.get(&target_asset.symbol);
        position
            .map(|p| p.average_entry_price.clone() * p.quantity.clone())
            .unwrap_or(Num::from(0))
    }

    // fn order_amount(&self, state: StrategyState, target_asset: TargetAsset) -> Num {
    //     let order = state.open_orders.get(&target_asset.symbol);
    //     order
    //         .map(|o| o.limit_price.clone().unwrap() )
    //         .unwrap_or(Num::from(0))
    // }
}

impl DoRule for DoBuyRule {
    fn evaluate(&self, state: &StrategyState, when_result: WhenResult) -> DoResult {
        DoResult {
            actions: vec![],
            skipped: false,
            skip_reason: Default::default(),
        }
    }
}
