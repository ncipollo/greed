use crate::strategy::state::StrategyState;
use crate::strategy::target::TargetAsset;

pub trait ForRule {
    fn evaluate(&self, state: &StrategyState) -> ForResult;
}

#[derive(Debug, Default, PartialEq)]
pub struct ForResult {
    target_assets: Vec<TargetAsset>,
}
