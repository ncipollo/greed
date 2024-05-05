use crate::num::NumFromFloat;
use crate::strategy::r#for::ForResult;
use crate::strategy::state::StrategyState;
use crate::strategy::target::TargetAsset;
use crate::strategy::when::{WhenResult, WhenRule};
use num_decimal::Num;

#[derive(Debug, Default, PartialEq)]
pub struct WhenGainAboveRule {
    gain_above_percent: f64,
}

impl WhenGainAboveRule {
    pub fn boxed(gain_above_percent: f64) -> Box<dyn WhenRule> {
        Box::new(Self { gain_above_percent })
    }

    fn is_gain_above(&self, state: &StrategyState, target_asset: &TargetAsset) -> bool {
        if self.is_state_valid(state, target_asset) {
            return false;
        }

        let position = &state.positions[&target_asset.symbol];
        let gain = position.unrealized_gain_total_percent.clone();
        gain.map(|g| g  >= Num::from_f64(self.gain_above_percent))
            .unwrap_or(false)
    }

    fn is_state_valid(&self, state: &StrategyState, target_asset: &TargetAsset) -> bool {
        !state.positions.contains_key(&target_asset.symbol)
    }
}

impl WhenRule for WhenGainAboveRule {
    fn evaluate(&self, state: &StrategyState, for_result: ForResult) -> WhenResult {
        let assets_above_gain = for_result
            .target_assets
            .iter()
            .filter(|target_asset| self.is_gain_above(state, target_asset))
            .cloned()
            .collect::<Vec<_>>();
        WhenResult {
            conditions_satisfied: !assets_above_gain.is_empty(),
            target_assets: assets_above_gain,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::AssetSymbol;
    use crate::platform::position::Position;
    use std::collections::HashMap;

    #[test]
    fn evaluate_no_positions() {
        let state = StrategyState {
            positions: vec![].into_iter().collect(),
            ..StrategyState::fixture()
        };
        let rule = WhenGainAboveRule::boxed(0.0);
        let target_assets = target_assets();
        let for_result = ForResult {
            target_assets: target_assets.clone(),
        };
        let result = rule.evaluate(&state, for_result);
        let expected = WhenResult {
            conditions_satisfied: false,
            target_assets: vec![],
        };
        assert_eq!(expected, result);
    }

    #[test]
    fn evaluate_no_gain() {
        let spy = AssetSymbol::new("SPY");
        let position = Position {
            unrealized_gain_today_percent: None,
            ..Position::fixture(spy.clone())
        };
        let state = StrategyState {
            positions: HashMap::from([(spy.clone(), position)]),
            ..StrategyState::fixture()
        };
        let rule = WhenGainAboveRule::boxed(0.0);
        let target_assets = target_assets();
        let for_result = ForResult {
            target_assets: target_assets.clone(),
        };
        let result = rule.evaluate(&state, for_result);
        let expected = WhenResult {
            conditions_satisfied: false,
            target_assets: vec![],
        };
        assert_eq!(expected, result);
    }

    #[test]
    fn evaluate_not_satisfied() {
        let spy = AssetSymbol::new("SPY");
        let position = Position {
            unrealized_gain_today_percent: Some(Num::from_f64(0.1)),
            ..Position::fixture(spy.clone())
        };
        let state = StrategyState {
            positions: HashMap::from([(spy.clone(), position)]),
            ..StrategyState::fixture()
        };
        let rule = WhenGainAboveRule::boxed(11.0);
        let target_assets = target_assets();
        let for_result = ForResult {
            target_assets: target_assets.clone(),
        };
        let result = rule.evaluate(&state, for_result);
        let expected = WhenResult {
            conditions_satisfied: false,
            target_assets: vec![],
        };
        assert_eq!(expected, result);
    }

    #[test]
    fn evaluate_satisfied() {
        let spy = AssetSymbol::new("SPY");
        let position = Position {
            unrealized_gain_total_percent: Some(Num::from_f64(10.0)),
            ..Position::fixture(spy.clone())
        };
        let state = StrategyState {
            positions: HashMap::from([(spy.clone(), position)]),
            ..StrategyState::fixture()
        };
        let rule = WhenGainAboveRule::boxed(10.0);
        let target_assets = target_assets();
        let for_result = ForResult {
            target_assets: target_assets.clone(),
        };
        let result = rule.evaluate(&state, for_result);
        let expected = WhenResult {
            conditions_satisfied: true,
            target_assets: vec![TargetAsset::full_percent(AssetSymbol::new("SPY"))],
        };
        assert_eq!(expected, result);
    }

    fn target_assets() -> Vec<TargetAsset> {
        vec![
            TargetAsset::full_percent(AssetSymbol::new("SPY")),
            TargetAsset::full_percent(AssetSymbol::new("VTI")),
        ]
    }
}
