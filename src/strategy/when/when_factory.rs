use crate::config::strategy::when::WhenConfig;
use crate::strategy::null::NullRule;
use crate::strategy::when::when_always::WhenAlwaysRule;
use crate::strategy::when::WhenRule;

pub struct WhenFactory;

impl WhenFactory {
    pub fn create_rule(config: WhenConfig) -> Box<dyn WhenRule> {
        match config {
            WhenConfig::AllOf { .. } => NullRule::when_boxed(),
            WhenConfig::Always { .. } => WhenAlwaysRule::boxed(),
            WhenConfig::BelowOneDay { .. } => NullRule::when_boxed(),
            WhenConfig::GainAbove { .. } => NullRule::when_boxed(),
            WhenConfig::Never { .. } => NullRule::when_boxed(),
        }
    }
}
