use crate::strategy::r#for::ForResult;
use crate::strategy::state::StrategyState;
use crate::strategy::target::TargetAsset;

pub trait WhenRule {
    fn evaluate(&self, state: &StrategyState, for_result: ForResult) -> WhenResult;
}

#[derive(Debug, Default, PartialEq)]
pub struct WhenResult {
    conditions_satisfied: bool,
    target_assets: Vec<TargetAsset>,
}
