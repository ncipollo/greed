use crate::num::NumPercent;
use crate::strategy::action::Action;
use crate::strategy::r#do::{DoResult, DoRule};
use crate::strategy::state::StrategyState;
use crate::strategy::target::TargetAsset;
use crate::strategy::when::WhenResult;
use num_decimal::Num;
use crate::strategy::skip::SkipReason;

pub struct DoBuyRule {
    buy_percent: f64,
}

impl DoBuyRule {
    pub fn boxed(buy_percent: f64) -> Box<dyn DoRule> {
        Box::new(Self { buy_percent })
    }

    fn actions(&self, state: &StrategyState, assets: &Vec<TargetAsset>) -> Vec<Action> {
        assets
            .iter()
            .filter_map(|asset| {
                let amount = self.calculate_buy_amount(state, asset);
                if amount > Num::from(0) {
                    Some(Action::buy_notional(asset.symbol.clone(), amount))
                } else {
                    None
                }
            })
            .collect()
    }

    fn calculate_buy_amount(&self, state: &StrategyState, target_asset: &TargetAsset) -> Num {
        let cash = state.account.cash.clone();
        let equity = state.account.equity.clone();
        let target_percent = self.target_percent(target_asset);

        let position_value = self.position_value(state, target_asset);
        let open_order_value = state.open_order_value(&target_asset.symbol);
        let desired_value = equity.percent_of(target_percent);

        let total_amount_notational = desired_value - position_value - open_order_value;
        total_amount_notational.clamp(Num::from(0), cash)
    }

    fn target_percent(&self, target_asset: &TargetAsset) -> f64 {
        (target_asset.percent * self.buy_percent) / 100.0
    }

    fn position_value(&self, state: &StrategyState, target_asset: &TargetAsset) -> Num {
        let position = state.positions.get(&target_asset.symbol);
        position
            .map(|p| p.average_entry_price.clone() * p.quantity.clone())
            .unwrap_or(Num::from(0))
    }
}

impl DoRule for DoBuyRule {
    fn evaluate(&self, state: &StrategyState, when_result: WhenResult) -> DoResult {
        let actions = self.actions(state, &when_result.target_assets);
        let skipped = actions.is_empty();
        DoResult {
            actions,
            skipped,
            skip_reason: SkipReason::NoTargetAssets,
        }
    }
}
