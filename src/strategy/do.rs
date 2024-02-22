use crate::strategy::state::StrategyState;
use crate::strategy::when::WhenResult;

pub trait DoRule {
    fn evaluate(&self, state: &StrategyState, when_result: WhenResult) -> DoResult;
}

#[derive(Debug, Default, PartialEq)]
pub struct DoResult;