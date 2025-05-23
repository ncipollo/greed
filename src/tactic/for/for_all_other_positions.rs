use crate::tactic::r#for::other_positions_filter::filter_assets_not_in_config;
use crate::tactic::r#for::{ForResult, ForRule};
use crate::tactic::state::TacticState;
use crate::tactic::target::TargetAsset;

#[derive(Debug, Default, PartialEq)]
pub struct ForAllOtherPositionsRule;

impl ForAllOtherPositionsRule {
    pub fn boxed() -> Box<dyn ForRule> {
        Box::new(Self)
    }
}

impl ForRule for ForAllOtherPositionsRule {
    fn evaluate(&self, state: &TacticState) -> ForResult {
        // Filter assets not in positions
        let filtered_assets = filter_assets_not_in_config(state);
        
        if filtered_assets.is_empty() {
            return ForResult { target_assets: vec![] };
        }

        // Set percent to 100% for each asset
        let target_assets = filtered_assets
            .into_iter()
            .map(|symbol| TargetAsset {
                symbol,
                percent: 100.0,
            })
            .collect();

        ForResult { target_assets }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::AssetSymbol;
    use crate::platform::position::Position;
    use std::collections::HashMap;

    #[test]
    fn evaluate_empty_positions() {
        let state = TacticState::default();
        let rule = ForAllOtherPositionsRule::boxed();
        let result = rule.evaluate(&state);
        let expected = ForResult {
            target_assets: vec![],
        };
        assert_eq!(expected, result);
    }

    #[test]
    fn evaluate_with_positions() {
        let spy = AssetSymbol::new("SPY");
        let vti = AssetSymbol::new("VTI");
        let qqq = AssetSymbol::new("QQQ");
        let bond = AssetSymbol::new("BOND"); // This will be a "rogue" position not in config
        
        let mut positions = HashMap::new();
        positions.insert(spy.clone(), Position::fixture(spy.clone()));
        positions.insert(bond.clone(), Position::fixture(bond.clone())); // Position not in config
        
        let all_assets = vec![spy.clone(), vti.clone(), qqq.clone()]; // Config doesn't include BOND
        
        let state = TacticState {
            positions,
            all_assets,
            ..TacticState::default()
        };
        
        let rule = ForAllOtherPositionsRule::boxed();
        let result = rule.evaluate(&state);
        
        for asset in &result.target_assets {
            assert_eq!(100.0, asset.percent);
        }
        
        let symbols: Vec<AssetSymbol> = result.target_assets.iter()
            .map(|asset| asset.symbol.clone())
            .collect();
        let expected_symbols = vec![bond.clone()];
        assert_eq!(expected_symbols, symbols);
    }
} 