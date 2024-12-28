use crate::config::tactic::when::WhenConfig;
use crate::tactic::null::NullRule;
use crate::tactic::when::when_all_of::WhenAllOfRule;
use crate::tactic::when::when_always::WhenAlwaysRule;
use crate::tactic::when::when_below_median::WhenBelowMedianRule;
use crate::tactic::when::when_gain_above::WhenGainAboveRule;
use crate::tactic::when::WhenRule;

pub struct WhenFactory;

impl WhenFactory {
    pub fn create_rule(config: WhenConfig) -> Box<dyn WhenRule> {
        match config {
            WhenConfig::AllOf { all_off } => {
                let rules = all_off.into_iter().map(Self::create_rule).collect();
                WhenAllOfRule::boxed(rules)
            }
            WhenConfig::Always { .. } => WhenAlwaysRule::boxed(),
            WhenConfig::BelowMedian {
                below_median_percent,
                median_period,
            } => WhenBelowMedianRule::boxed(below_median_percent, median_period),
            WhenConfig::GainAbove { gain_above_percent } => {
                WhenGainAboveRule::boxed(gain_above_percent)
            }
            WhenConfig::Never { .. } => NullRule::when_boxed(),
        }
    }
}
