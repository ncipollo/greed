mod when_all_of;
pub mod when_always;
mod when_below_median;
pub mod when_factory;
mod when_gain_above;

use crate::tactic::r#for::ForResult;
use crate::tactic::state::TacticState;
use crate::tactic::target::TargetAsset;

pub trait WhenRule {
    fn evaluate(&self, state: &TacticState, for_result: ForResult) -> WhenResult;
}

#[derive(Debug, Default, PartialEq)]
pub struct WhenResult {
    pub conditions_satisfied: bool,
    pub target_assets: Vec<TargetAsset>,
}
