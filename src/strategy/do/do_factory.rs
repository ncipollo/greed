use crate::config::strategy::r#do::DoConfig;
use crate::strategy::null::NullRule;
use crate::strategy::r#do::do_sellall::DoSellAllRule;
use crate::strategy::r#do::DoRule;

pub struct DoFactory;

impl DoFactory {
    pub fn create_rule(config: DoConfig) -> Box<dyn DoRule> {
        match config {
            DoConfig::Buy { .. } => NullRule::do_boxed(),
            DoConfig::SellAll { .. } => DoSellAllRule::boxed(),
        }
    }
}
