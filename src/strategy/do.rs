use crate::strategy::action::Action;
use crate::strategy::state::StrategyState;
use crate::strategy::when::WhenResult;

pub trait DoRule {
    fn evaluate(&self, state: &StrategyState, when_result: WhenResult) -> DoResult;
}

#[derive(Debug, Default, PartialEq)]
pub struct DoResult {
    pub actions: Vec<Action>,
    pub skipped: bool,
}

impl DoResult {
    pub fn skip() -> Self {
        Self {
            skipped: true,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skip() {
        let result = DoResult::skip();
        let expected = DoResult {
            skipped: true,
            ..Default::default()
        };
        assert_eq!(expected, result);
    }
}
