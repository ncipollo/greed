use crate::strategy::r#do::{DoResult, DoRule};
use crate::strategy::r#for::{ForResult, ForRule};
use crate::strategy::state::StrategyState;
use crate::strategy::when::{WhenResult, WhenRule};

#[derive(Debug, PartialEq)]
pub struct NullRule;

impl ForRule for NullRule {
    fn evaluate(&self, _: &StrategyState) -> ForResult {
        ForResult::default()
    }
}

impl WhenRule for NullRule {
    fn evaluate(&self, _: &StrategyState, _: ForResult) -> WhenResult {
        Default::default()
    }
}

impl DoRule for NullRule {
    fn evaluate(&self, _: &StrategyState, _: WhenResult) -> DoResult {
        DoResult::default()
    }
}

impl NullRule {
    pub fn for_boxed() -> Box<dyn ForRule> {
        Box::new(NullRule {})
    }

    pub fn when_boxed() -> Box<dyn WhenRule> {
        Box::new(NullRule {})
    }
}
