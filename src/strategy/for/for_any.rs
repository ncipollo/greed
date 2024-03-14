use crate::asset::AssetSymbol;
use crate::strategy::r#for::{ForResult, ForRule};
use crate::strategy::state::StrategyState;
use crate::strategy::target::TargetAsset;

#[derive(Debug, Default, PartialEq)]
pub struct ForAnyStockRule {
    stocks: Vec<AssetSymbol>,
}

impl ForAnyStockRule {
    pub fn boxed(stocks: Vec<AssetSymbol>) -> Box<dyn ForRule> {
        Box::new(Self { stocks })
    }
}

impl ForRule for ForAnyStockRule {
    fn evaluate(&self, _state: &StrategyState) -> ForResult {
        let percent = 100.0 / (self.stocks.len() as f64);
        let target_assets = self
            .stocks
            .iter()
            .map(|stock| TargetAsset {
                symbol: stock.clone(),
                percent,
            })
            .collect();
        ForResult { target_assets }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluate_empty_stocks() {
        let state: StrategyState = Default::default();
        let rule = ForAnyStockRule::boxed(vec![]);
        let result = rule.evaluate(&state);
        let expected = ForResult { target_assets: vec![] };
        assert_eq!(expected, result)
    }

    #[test]
    fn evaluate_one_stock() {
        let state: StrategyState = Default::default();
        let spy = AssetSymbol::new("SPY");
        let rule = ForAnyStockRule::boxed(vec![spy]);
        let result = rule.evaluate(&state);
        let expected = ForResult { target_assets: vec![
            TargetAsset::full_percent(AssetSymbol::new("SPY"))
        ] };
        assert_eq!(expected, result)
    }

    #[test]
    fn evaluate_multiple_stocks() {
        let state: StrategyState = Default::default();
        let spy = AssetSymbol::new("SPY");
        let vti = AssetSymbol::new("VTI");
        let vea = AssetSymbol::new("VEA");
        let rule = ForAnyStockRule::boxed(vec![spy, vti, vea]);
        let result = rule.evaluate(&state);
        let expected = ForResult { target_assets: vec![
            TargetAsset { symbol: AssetSymbol::new("SPY"), percent: 100.0/3.0 },
            TargetAsset { symbol: AssetSymbol::new("VTI"), percent: 100.0/3.0 },
            TargetAsset { symbol: AssetSymbol::new("VEA"), percent: 100.0/3.0 },
        ] };
        assert_eq!(expected, result)
    }
}