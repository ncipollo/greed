use crate::float::FloatAmountRounding;
use crate::strategy::action::Action;
use crate::strategy::r#do::{DoResult, DoRule};
use crate::strategy::skip::SkipReason;
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
                let position_amount = position
                    .map(|p| p.quantity_available.clone())
                    .unwrap_or_default();
                // We need to round down after 7 significant digits because anything more than that
                // does not serialize correctly in num.
                let sell_amount = target_asset
                    .apply_percent(position_amount)
                    .round_for_quantity();
                Action::sell_quantity(symbol.clone(), sell_amount)
            })
            .filter(|a| !a.is_empty())
            .collect::<Vec<_>>();
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
    use std::collections::HashMap;

    use crate::asset::AssetSymbol;
    use crate::platform::position::Position;
    use crate::strategy::target::TargetAsset;

    use super::*;

    #[test]
    fn evaluate() {
        let rule = DoSellAllRule::boxed();
        let state = create_state();
        let when_result = create_when_result();
        let do_result = rule.evaluate(&state, when_result);
        let expected_actions = vec![
            Action::sell_quantity(AssetSymbol::new("SPY"), 50.0),
            Action::sell_quantity(AssetSymbol::new("VTI"), 25.0),
        ];
        let expected = DoResult {
            actions: expected_actions,
            skip_reason: SkipReason::NoTargetAssets,
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
        let expected = DoResult::skip(SkipReason::NoTargetAssets);
        assert_eq!(expected, do_result)
    }

    #[test]
    fn evaluate_empty_when() {
        let rule = DoSellAllRule::boxed();
        let state = create_state();
        let when_result = WhenResult::default();
        let do_result = rule.evaluate(&state, when_result);
        let expected = DoResult::skip(SkipReason::NoTargetAssets);
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
