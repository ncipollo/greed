use crate::strategy::action::Action;
use crate::strategy::r#do::{DoResult, DoRule};
use crate::strategy::state::StrategyState;
use crate::strategy::when::WhenResult;

pub struct DoSellAllRule;

impl DoSellAllRule {
    pub fn boxed() -> Box<dyn DoRule> {
        Box::new(Self {})
    }
}

impl DoRule for DoSellAllRule {
    fn evaluate(&self, state: &StrategyState, when_result: WhenResult) -> DoResult {
        let actions = when_result
            .target_assets
            .into_iter()
            .map(|target_asset| {
                let symbol = &target_asset.symbol;
                let position = state.positions.get(symbol);
                let position_amount = position.map(|p| p.quantity.clone()).unwrap_or_default();
                let sell_amount = target_asset.apply_percent(position_amount);
                Action::sell_notional(symbol.clone(), sell_amount)
            })
            .filter(|a| !a.is_empty())
            .collect::<Vec<_>>();
        DoResult {
            actions,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::AssetSymbol;
    use crate::platform::position::Position;
    use crate::strategy::target::TargetAsset;
    use num_decimal::Num;
    use std::collections::HashMap;

    #[test]
    fn evaluate() {
        let rule = DoSellAllRule::boxed();
        let state = create_state();
        let when_result = create_when_result();
        let do_result = rule.evaluate(&state, when_result);
        let expected_actions = vec![
            Action::sell_notional(AssetSymbol::new("SPY"), Num::from(50)),
            Action::sell_notional(AssetSymbol::new("VTI"), Num::from(25)),
        ];
        let expected = DoResult {
            actions: expected_actions,
            ..Default::default()
        };
        assert_eq!(expected, do_result)
    }

    #[test]
    fn evaluate_empty_state() {
        let rule = DoSellAllRule::boxed();
        let state = StrategyState::default();
        let when_result = create_when_result();
        let do_result = rule.evaluate(&state, when_result);
        let expected = DoResult {
            actions: vec![],
            ..Default::default()
        };
        assert_eq!(expected, do_result)
    }

    #[test]
    fn evaluate_empty_when() {
        let rule = DoSellAllRule::boxed();
        let state = create_state();
        let when_result = WhenResult::default();
        let do_result = rule.evaluate(&state, when_result);
        let expected = DoResult {
            actions: vec![],
            ..Default::default()
        };
        assert_eq!(expected, do_result)
    }

    fn create_state() -> StrategyState {
        let spy_position = Position::fixture(AssetSymbol::new("SPY"));
        let vti_position = Position::fixture(AssetSymbol::new("VTI"));
        let positions = HashMap::from([
            (AssetSymbol::new("SPY"), spy_position),
            (AssetSymbol::new("VTI"), vti_position),
        ]);
        StrategyState {
            positions,
            ..Default::default()
        }
    }

    fn create_when_result() -> WhenResult {
        let spy_asset = TargetAsset::full_percent(AssetSymbol::new("SPY"));
        let vti_asset = TargetAsset {
            symbol: AssetSymbol::new("VTI"),
            percent: 50.0,
        };
        WhenResult {
            conditions_satisfied: true,
            target_assets: vec![spy_asset, vti_asset],
        }
    }
}
