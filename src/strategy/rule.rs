use crate::config::strategy::rule::RuleConfig;
use crate::config::strategy::StrategyConfig;
use crate::lowercase_enum_display;
use crate::strategy::null::NullRule;
use crate::strategy::r#do::{DoResult, DoRule};
use crate::strategy::r#do::do_factory::DoFactory;
use crate::strategy::r#for::for_factory::ForFactory;
use crate::strategy::r#for::ForRule;
use crate::strategy::skip::SkipReason;
use crate::strategy::state::StrategyState;
use crate::strategy::when::when_factory::WhenFactory;
use crate::strategy::when::WhenRule;

pub struct RuleSet {
    for_rule: Box<dyn ForRule>,
    when_rule: Box<dyn WhenRule>,
    do_rule: Box<dyn DoRule>
}

impl Default for RuleSet {
    fn default() -> Self {
        Self {
            for_rule: Box::new(NullRule),
            when_rule: Box::new(NullRule),
            do_rule: Box::new(NullRule)
        }
    }
}

impl RuleSet {
    pub fn from_config(rule_config: RuleConfig) -> Self {
        Self {
            for_rule: ForFactory::create_rule(rule_config.for_config),
            when_rule: WhenFactory::create_rule(rule_config.when_config),
            do_rule: DoFactory::create_rule(rule_config.do_config)
        }
    }

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

pub struct StrategyRuleset {
    pub buy: RuleSet,
    pub sell: RuleSet
}

impl StrategyRuleset {
    pub fn from_config(strategy_config: StrategyConfig) -> Self {
        Self {
            buy: RuleSet::from_config(strategy_config.buy),
            sell: RuleSet::from_config(strategy_config.sell)
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum RuleType {
    Buy,
    Sell,
}

lowercase_enum_display!(RuleType);

#[cfg(test)]
mod tests {
    use crate::strategy::r#do::do_sellall::DoSellAllRule;
    use crate::strategy::r#for::for_stock::ForStockRule;
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
        let expected = DoResult {
            actions: vec![],
            skipped: true,
            skip_reason: SkipReason::NoTargetAssets
        };
        assert_eq!(expected, result);
    }
}
