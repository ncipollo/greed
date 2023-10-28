use crate::config::rules::merge::ConfigRuleMerge;
use serde::{Deserialize, Serialize};

/// A rule which indicates what percent of the user's portfolio we want this asset or strategy to target.
/// When merged with higher configuration rules they will multiply together. So if the strategy has
/// a target of 50% and the stock has a target of 10%, then the resulting target would be 5%.
#[derive(Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct PortfolioTargetRule {
    /// Percent out of 100
    pub percent: f32,
}

impl ConfigRuleMerge for PortfolioTargetRule {
    fn merge(&self, higher: Option<Self>) -> Self {
        let higher_percent = higher.map(|h| h.percent).unwrap_or(100.0);
        Self {
            percent: (higher_percent * self.percent) / 100.0,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::config::rules::merge::ConfigRuleMerge;
    use crate::config::rules::target::PortfolioTargetRule;

    #[test]
    fn merge_no_higher() {
        let lower = PortfolioTargetRule { percent: 50.0 };
        let merged = lower.merge(None);
        assert_eq!(merged, lower);
    }

    #[test]
    fn merge_with_higher() {
        let higher = PortfolioTargetRule { percent: 50.0 };
        let lower = PortfolioTargetRule { percent: 50.0 };
        let merged = lower.merge(Some(higher));
        let expected = PortfolioTargetRule { percent: 25.0 };
        assert_eq!(merged, expected);
    }
}
