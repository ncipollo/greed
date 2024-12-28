use crate::tactic::r#for::ForResult;
use crate::tactic::state::TacticState;
use crate::tactic::when::{WhenResult, WhenRule};

pub struct WhenAlwaysRule;

impl WhenRule for WhenAlwaysRule {
    fn evaluate(&self, _state: &TacticState, for_result: ForResult) -> WhenResult {
        WhenResult {
            conditions_satisfied: true,
            target_assets: for_result.target_assets,
        }
    }
}

impl WhenAlwaysRule {
    pub fn boxed() -> Box<dyn WhenRule> {
        Box::new(WhenAlwaysRule {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tactic::target::TargetAsset;

    #[test]
    fn evaluate() {
        let state = TacticState::default();
        let for_result = ForResult::fixture();
        let rule = WhenAlwaysRule {};
        let result = rule.evaluate(&state, for_result);
        let expected = WhenResult {
            conditions_satisfied: true,
            target_assets: vec![TargetAsset::fixture()],
        };
        assert_eq!(expected, result)
    }
}
