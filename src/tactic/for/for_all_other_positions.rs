use crate::tactic::r#for::for_assets_not_in_config::filter_assets_not_in_config;
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
        
        let mut positions = HashMap::new();
        positions.insert(spy.clone(), Position::fixture(spy.clone()));
        
        let all_assets = vec![spy.clone(), vti.clone(), qqq.clone()];
        
        let state = TacticState {
            positions,
            all_assets,
            ..TacticState::default()
        };
        
        let rule = ForAllOtherPositionsRule::boxed();
        let result = rule.evaluate(&state);
        
        // We should have 2 assets (VTI and QQQ) not in positions
        assert_eq!(2, result.target_assets.len());
        
        // All assets should have 100% percent
        for asset in &result.target_assets {
            assert_eq!(100.0, asset.percent);
        }
        
        let symbols: Vec<AssetSymbol> = result.target_assets.iter()
            .map(|asset| asset.symbol.clone())
            .collect();
        assert!(!symbols.contains(&spy));
        assert!(symbols.contains(&vti));
        assert!(symbols.contains(&qqq));
    }
} 