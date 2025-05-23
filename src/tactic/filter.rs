use crate::asset::AssetSymbol;
use crate::tactic::state::TacticState;

pub fn filter_assets_not_in_config(state: &TacticState) -> Vec<AssetSymbol> {
    // Get assets from current positions
    let position_assets: Vec<AssetSymbol> = state.positions.keys().cloned().collect();
    
    // Filter out those that exist in our configured assets (all_assets)
    position_assets
        .iter()
        .filter(|asset| !state.all_assets.contains(*asset))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::position::Position;
    use std::collections::HashMap;

    #[test]
    fn test_filter_assets_not_in_config() {
        // Setup test data
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
        
        // Run the function
        let result = filter_assets_not_in_config(&state);
        
        // Verify results - should return position assets not in config
        assert_eq!(1, result.len());
        assert!(!result.contains(&spy)); // SPY is in config, so filtered out
        assert!(result.contains(&bond)); // BOND is not in config, so included
        assert!(!result.contains(&vti)); // VTI is not a position
        assert!(!result.contains(&qqq)); // QQQ is not a position
    }
} 