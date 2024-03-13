use crate::strategy::null::NullRule;
use crate::strategy::r#do::{DoResult, DoRule};
use crate::strategy::r#for::ForRule;
use crate::strategy::skip::SkipReason;
use crate::strategy::state::StrategyState;
use crate::strategy::when::WhenRule;

struct RuleSet {
    for_rule: Box<dyn ForRule>,
    do_rule: Box<dyn DoRule>,
    when_rule: Box<dyn WhenRule>,
}

impl Default for RuleSet {
    fn default() -> Self {
        Self {
            for_rule: Box::new(NullRule),
            do_rule: Box::new(NullRule),
            when_rule: Box::new(NullRule),
        }
    }
}

impl RuleSet {
    pub fn evaluate(&self, state: &StrategyState) -> DoResult {
        let for_result = self.for_rule.evaluate(state);
        if for_result.is_empty() {
            return DoResult::skip(SkipReason::NoTargetAssets);
        }

        let when_result = self.when_rule.evaluate(state, for_result);
        if !when_result.conditions_satisfied {
            return DoResult::skip(SkipReason::ConditionsUnsatisfied);
        }

        self.do_rule.evaluate(state, when_result)
    }
}

#[cfg(test)]
mod tests {
    use crate::strategy::action::Action;
    use crate::strategy::r#do::do_sellall::DoSellAllRule;
    use crate::strategy::r#for::for_stock::ForStockRule;
    use crate::strategy::target::TargetAsset;
    use crate::strategy::when::when_always::WhenAlwaysRule;
    use super::*;

    #[test]
    fn evaluate_skip_no_assets() {
        let rule_set = RuleSet::default();
        let state = StrategyState::default();
        let result = rule_set.evaluate(&state);
        assert_eq!(DoResult::skip(SkipReason::NoTargetAssets), result);
    }

    #[test]
    fn evaluate_skip_conditions_unsatisfied() {
        let rule_set =  RuleSet {
            for_rule: ForStockRule::boxed("SPY"),
            ..Default::default()
        };
        let state = StrategyState::default();
        let result = rule_set.evaluate(&state);
        assert_eq!(DoResult::skip(SkipReason::ConditionsUnsatisfied), result);
    }

    #[test]
    fn evaluate_successful() {
        let rule_set =  RuleSet {
            for_rule: ForStockRule::boxed("SPY"),
            when_rule: WhenAlwaysRule::boxed(),
            do_rule: DoSellAllRule::boxed(),
        };
        let state = StrategyState::default();
        let result = rule_set.evaluate(&state);
        assert_eq!(DoResult::default(), result);
    }
}
