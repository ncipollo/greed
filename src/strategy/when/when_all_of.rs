use crate::strategy::r#for::ForResult;
use crate::strategy::state::StrategyState;
use crate::strategy::when::{WhenResult, WhenRule};

pub struct WhenAllOfRule {
    rules: Vec<Box<dyn WhenRule>>,
}

impl WhenAllOfRule {
    pub fn boxed(rules: Vec<Box<dyn WhenRule>>) -> Box<dyn WhenRule> {
        Box::new(Self { rules })
    }
}

impl WhenRule for WhenAllOfRule {
    fn evaluate(&self, state: &StrategyState, for_result: ForResult) -> WhenResult {
        let filtered_assets = for_result
            .target_assets
            .into_iter()
            .filter(|t| {
                self.rules.iter().all(|rule| {
                    let asset_result = ForResult {
                        target_assets: vec![t.clone()],
                    };
                    rule.evaluate(state, asset_result).conditions_satisfied
                })
            })
            .collect::<Vec<_>>();
        WhenResult {
            conditions_satisfied: !filtered_assets.is_empty(),
            target_assets: filtered_assets,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::asset::AssetSymbol;
    use crate::strategy::null::NullRule;
    use crate::strategy::r#for::ForResult;
    use crate::strategy::state::StrategyState;
    use crate::strategy::target::TargetAsset;
    use crate::strategy::when::when_all_of::WhenAllOfRule;
    use crate::strategy::when::when_always::WhenAlwaysRule;
    use crate::strategy::when::WhenResult;

    #[test]
    fn evaluate_not_satisfied() {
        let state = StrategyState::fixture();
        let rule = WhenAllOfRule::boxed(vec![WhenAlwaysRule::boxed(), NullRule::when_boxed()]);
        let target_assets = target_assets();
        let for_result = ForResult {
            target_assets: target_assets.clone(),
        };
        let result = rule.evaluate(&state, for_result);
        let expected = WhenResult {
            conditions_satisfied: false,
            target_assets: vec![],
        };
        assert_eq!(expected, result)
    }

    #[test]
    fn evaluate_satisfied() {
        let state = StrategyState::fixture();
        let rule = WhenAllOfRule::boxed(vec![WhenAlwaysRule::boxed()]);
        let target_assets = target_assets();
        let for_result = ForResult {
            target_assets: target_assets.clone(),
        };
        let result = rule.evaluate(&state, for_result);
        let expected = WhenResult {
            conditions_satisfied: true,
            target_assets: target_assets.clone(),
        };
        assert_eq!(expected, result)
    }

    fn target_assets() -> Vec<TargetAsset> {
        vec![
            TargetAsset::full_percent(AssetSymbol::new("SPY")),
            TargetAsset::full_percent(AssetSymbol::new("VTI")),
        ]
    }
}
