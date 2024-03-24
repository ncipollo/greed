mod when_all_of;
pub mod when_always;
mod when_below_median;
pub mod when_factory;
mod when_gain_above;

use crate::strategy::r#for::ForResult;
use crate::strategy::state::StrategyState;
use crate::strategy::target::TargetAsset;

pub trait WhenRule {
    fn evaluate(&self, state: &StrategyState, for_result: ForResult) -> WhenResult;
}

#[derive(Debug, Default, PartialEq)]
pub struct WhenResult {
    pub conditions_satisfied: bool,
    pub target_assets: Vec<TargetAsset>,
}
