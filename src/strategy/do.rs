pub mod do_factory;
pub mod do_sellall;
mod do_buy;

use crate::strategy::action::Action;
use crate::strategy::skip::SkipReason;
use crate::strategy::state::StrategyState;
use crate::strategy::when::WhenResult;

pub trait DoRule {
    fn evaluate(&self, state: &StrategyState, when_result: WhenResult) -> DoResult;
}

#[derive(Debug, Default, PartialEq)]
pub struct DoResult {
    pub actions: Vec<Action>,
    pub skipped: bool,
    pub skip_reason: SkipReason,
}

impl DoResult {
    pub fn skip(reason: SkipReason) -> Self {
        Self {
            skipped: true,
            skip_reason: reason,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skip() {
        let result = DoResult::skip(SkipReason::NoTargetAssets);
        let expected = DoResult {
            skipped: true,
            skip_reason: SkipReason::NoTargetAssets,
            ..Default::default()
        };
        assert_eq!(expected, result);
    }
}
