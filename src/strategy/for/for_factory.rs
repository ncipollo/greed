use crate::config::strategy::r#for::ForConfig;
use crate::strategy::null::NullRule;
use crate::strategy::r#for::for_any::ForAnyStockRule;
use crate::strategy::r#for::for_stock::ForStockRule;
use crate::strategy::r#for::ForRule;

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
