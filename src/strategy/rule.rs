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

mod tests {}
