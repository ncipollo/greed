use crate::asset::AssetSymbol;
use crate::strategy::r#for::{ForResult, ForRule};
use crate::strategy::state::StrategyState;
use crate::strategy::target::TargetAsset;

#[derive(Debug, Default, PartialEq)]
pub struct ForStockRule {
    pub stock: AssetSymbol,
}

impl ForRule for ForStockRule {
    fn evaluate(&self, _state: &StrategyState) -> ForResult {
        let target = TargetAsset::full_percent(self.stock.clone());
        ForResult {
            target_assets: vec![target],
        }
    }
}

impl ForStockRule {
    pub fn boxed<T: Into<AssetSymbol>>(stock: T) -> Box<dyn ForRule> {
        Box::new(Self {
            stock: stock.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluate() {
        let state: StrategyState = Default::default();
        let rule = ForStockRule::boxed("SPY");
        let result = rule.evaluate(&state);
        let expected = ForResult {
            target_assets: vec![TargetAsset::full_percent(AssetSymbol::new("SPY"))],
        };
        assert_eq!(expected, result)
    }
}
