use crate::strategy::r#do::{DoResult, DoRule};
use crate::strategy::r#for::{ForResult, ForRule};
use crate::strategy::state::StrategyState;
use crate::strategy::when::{WhenResult, WhenRule};

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