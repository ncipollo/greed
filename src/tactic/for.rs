mod for_all_other_positions;
mod for_any;
pub mod for_factory;
pub mod for_stock;

use crate::tactic::state::TacticState;
use crate::tactic::target::TargetAsset;
use std::fmt::Debug;

pub trait ForRule {
    fn evaluate(&self, state: &TacticState) -> ForResult;
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ForResult {
    pub target_assets: Vec<TargetAsset>,
}

impl ForResult {
    pub fn is_empty(&self) -> bool {
        self.target_assets.is_empty()
    }

    #[cfg(test)]
    pub fn fixture() -> Self {
        Self {
            target_assets: vec![TargetAsset::fixture()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_result_default() {
        assert_eq!(
            ForResult {
                target_assets: vec![]
            },
            Default::default()
        )
    }

    #[test]
    fn for_result_is_empty() {
        let for_result = ForResult {
            target_assets: vec![],
        };
        assert!(for_result.is_empty());
    }

    #[test]
    fn for_result_is_not_empty() {
        let for_result = ForResult {
            target_assets: vec![TargetAsset::default()],
        };
        assert!(!for_result.is_empty())
    }
}
