use crate::strategy::r#do::{DoResult, DoRule};
use crate::strategy::state::StrategyState;
use crate::strategy::target::TargetAsset;
use crate::strategy::when::WhenResult;

pub struct DoBuyRule {
    buy_percent: f64
}

impl DoBuyRule {
    pub fn boxed(buy_percent: f64) -> Box<dyn DoRule> {
        Box::new(Self { buy_percent })
    }

    fn calculate_buy_amount(&self, state: &StrategyState, target_asset: &TargetAsset) -> f64 {
        // Get position, get orders, and get account
        // total desired amount = equity * target percent
        // buy amount  = total desired position - postion - pending orders
        0.0
    }
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