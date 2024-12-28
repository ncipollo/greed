use crate::config::tactic::r#for::ForConfig;
use crate::tactic::null::NullRule;
use crate::tactic::r#for::for_any::ForAnyStockRule;
use crate::tactic::r#for::for_stock::ForStockRule;
use crate::tactic::r#for::ForRule;

pub struct ForFactory;

impl ForFactory {
    pub fn create_rule(config: ForConfig) -> Box<dyn ForRule> {
        match config {
            ForConfig::AnyOf { any_of } => ForAnyStockRule::boxed(any_of),
            ForConfig::Nothing { .. } => NullRule::for_boxed(),
            ForConfig::Stock { stock } => ForStockRule::boxed(stock),
        }
    }
}
