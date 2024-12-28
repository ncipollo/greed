use crate::config::tactic::r#do::DoConfig;
use crate::tactic::r#do::do_buy::DoBuyRule;
use crate::tactic::r#do::do_sellall::DoSellAllRule;
use crate::tactic::r#do::DoRule;

pub struct DoFactory;

impl DoFactory {
    pub fn create_rule(config: DoConfig) -> Box<dyn DoRule> {
        match config {
            DoConfig::Buy { buy_percent } => DoBuyRule::boxed(buy_percent),
            DoConfig::SellAll { .. } => DoSellAllRule::boxed(),
        }
    }
}
