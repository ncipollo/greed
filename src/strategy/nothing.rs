use crate::strategy::r#for::{ForResult, ForRule};
use crate::strategy::state::StrategyState;

pub struct NothingRule;

impl ForRule for NothingRule {
    fn evaluate(&self, _: &StrategyState) -> ForResult {
        ForResult::default()
    }
}