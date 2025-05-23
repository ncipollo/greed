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
        let positions = &state.positions;
        if positions.is_empty() {
            return ForResult { target_assets: vec![] };
        }

        let percent = 100.0 / (positions.len() as f64);
        let target_assets = positions
            .keys()
            .map(|symbol| TargetAsset {
                symbol: symbol.clone(),
                percent,
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
        
        let mut positions = HashMap::new();
        positions.insert(spy.clone(), Position::fixture(spy.clone()));
        positions.insert(vti.clone(), Position::fixture(vti.clone()));
        
        let state = TacticState {
            positions,
            ..TacticState::default()
        };
        
        let rule = ForAllOtherPositionsRule::boxed();
        let result = rule.evaluate(&state);
        
        // We don't know the order, so check properties separately
        assert_eq!(2, result.target_assets.len());
        assert_eq!(50.0, result.target_assets[0].percent);
        assert_eq!(50.0, result.target_assets[1].percent);
        
        let symbols: Vec<AssetSymbol> = result.target_assets.iter()
            .map(|asset| asset.symbol.clone())
            .collect();
        assert!(symbols.contains(&spy));
        assert!(symbols.contains(&vti));
    }
} 