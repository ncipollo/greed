use crate::tactic::r#do::{DoResult, DoRule};
use crate::tactic::r#for::{ForResult, ForRule};
use crate::tactic::state::TacticState;
use crate::tactic::when::{WhenResult, WhenRule};

#[derive(Debug, PartialEq)]
pub struct NullRule;

impl ForRule for NullRule {
    fn evaluate(&self, _: &TacticState) -> ForResult {
        ForResult::default()
    }
}

impl WhenRule for NullRule {
    fn evaluate(&self, _: &TacticState, _: ForResult) -> WhenResult {
        Default::default()
    }
}

impl DoRule for NullRule {
    fn evaluate(&self, _: &TacticState, _: WhenResult) -> DoResult {
        DoResult::default()
    }
}

impl NullRule {
    pub fn do_boxed() -> Box<dyn DoRule> {
        Box::new(NullRule {})
    }
    
    pub fn for_boxed() -> Box<dyn ForRule> {
        Box::new(NullRule {})
    }

    pub fn when_boxed() -> Box<dyn WhenRule> {
        Box::new(NullRule {})
    }
}
