use crate::config::strategy::r#for::ForConfig;
use crate::strategy::null::NullRule;
use crate::strategy::r#for::for_stock::ForStockRule;
use crate::strategy::r#for::ForRule;

pub struct ForFactory;

impl ForFactory {
    pub fn create_rule(config: ForConfig) -> Box<dyn ForRule> {
        match config {
            ForConfig::Nothing { .. } => Box::new(NullRule {}),
            ForConfig::Stock { stock } => Box::new(ForStockRule { stock }),
        }
    }
}
