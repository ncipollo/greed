use crate::float::PercentOps;
use crate::strategy::action::Action;
use crate::strategy::r#do::{DoResult, DoRule};
use crate::strategy::skip::SkipReason;
use crate::strategy::state::StrategyState;
use crate::strategy::target::TargetAsset;
use crate::strategy::when::WhenResult;

pub struct DoBuyRule {
    buy_percent: f64,
}

impl DoBuyRule {
    pub fn boxed(buy_percent: f64) -> Box<dyn DoRule> {
        Box::new(Self { buy_percent })
    }

    fn actions(&self, state: &StrategyState, assets: &Vec<TargetAsset>) -> Vec<Action> {
        let mut remaining_cash = state.account.cash.clone();
        assets
            .iter()
            .filter_map(|asset| {
                let amount = self.calculate_buy_amount(state, asset, remaining_cash.clone());
                remaining_cash -= amount.clone();
                if amount > 0.0 {
                    Some(Action::buy_notional(asset.symbol.clone(), amount))
                } else {
                    None
                }
            })
            .collect()
    }

    fn calculate_buy_amount(
        &self,
        state: &StrategyState,
        target_asset: &TargetAsset,
        remaining_cash: f64,
    ) -> f64 {
        let equity = state.account.equity.clone();
        let target_percent = self.target_percent(target_asset);

        let position_value = self.position_value(state, target_asset);
        let open_order_value = state.open_order_value(&target_asset.symbol);
        let desired_value = equity.percent_of(target_percent);

        let total_amount_notational = desired_value - position_value - open_order_value;
        total_amount_notational.clamp(0.0, remaining_cash)
    }

    fn target_percent(&self, target_asset: &TargetAsset) -> f64 {
        (target_asset.percent * self.buy_percent) / 100.0
    }

    fn position_value(&self, state: &StrategyState, target_asset: &TargetAsset) -> f64 {
        let position = state.positions.get(&target_asset.symbol);
        position
            .map(|p| p.average_entry_price.clone() * p.quantity.clone())
            .unwrap_or(0.0)
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

#[cfg(test)]
mod tests {
    use crate::asset::AssetSymbol;
    use crate::platform::account::Account;

    use super::*;

    #[test]
    fn evaluate() {
        let rule = DoBuyRule::boxed(100.0);
        let state = StrategyState::fixture();
        let when_result = WhenResult {
            target_assets: vec![
                TargetAsset::new(AssetSymbol::new("VTI"), 50.0),
                TargetAsset::new(AssetSymbol::new("SPY"), 50.0),
            ],
            ..Default::default()
        };
        let result = rule.evaluate(&state, when_result);
        let expected = DoResult {
            actions: vec![
                Action::buy_notional(AssetSymbol::new("VTI"), 50.0),
                Action::buy_notional(AssetSymbol::new("SPY"), 50.0),
            ],
            skipped: false,
            skip_reason: SkipReason::NoTargetAssets,
        };
        assert_eq!(expected, result)
    }

    #[test]
    fn evaluate_clamped_to_zero() {
        let rule = DoBuyRule::boxed(-100.0);
        let state = StrategyState::fixture();
        let when_result = WhenResult {
            target_assets: vec![
                TargetAsset::new(AssetSymbol::new("VTI"), 50.0),
                TargetAsset::new(AssetSymbol::new("SPY"), 50.0),
            ],
            ..Default::default()
        };
        let result = rule.evaluate(&state, when_result);
        let expected = DoResult {
            actions: vec![],
            skipped: true,
            skip_reason: SkipReason::NoTargetAssets,
        };
        assert_eq!(expected, result)
    }

    #[test]
    fn evaluate_clamped_to_cash() {
        let rule = DoBuyRule::boxed(100.0);
        let state = StrategyState {
            account: Account {
                cash: 5.0,
                ..Account::fixture()
            },
            ..StrategyState::fixture()
        };
        let when_result = WhenResult {
            target_assets: vec![
                TargetAsset::new(AssetSymbol::new("VTI"), 50.0),
                TargetAsset::new(AssetSymbol::new("SPY"), 50.0),
            ],
            ..Default::default()
        };
        let result = rule.evaluate(&state, when_result);
        let expected = DoResult {
            actions: vec![
                Action::buy_notional(AssetSymbol::new("VTI"), 5.0)
            ],
            skipped: false,
            skip_reason: SkipReason::NoTargetAssets,
        };
        assert_eq!(expected, result)
    }
}
