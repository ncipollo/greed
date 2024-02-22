use crate::strategy::target::TargetAsset;

pub trait WhenRule {

}

#[derive(Debug, Default, PartialEq)]
pub struct WhenResult {
    conditions_satisfied: bool,
    target_assets: Vec<TargetAsset>,
}
