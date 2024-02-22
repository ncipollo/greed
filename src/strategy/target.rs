use crate::asset::AssetSymbol;

#[derive(Debug, PartialEq)]
pub struct TargetAsset {
    symbol: AssetSymbol,
    percent: f64,
}

impl Default for TargetAsset {
    fn default() -> Self {
        Self {
            symbol: AssetSymbol::default(),
            percent: 0.0,
        }
    }
}

mod tests {
    use super::*;

    #[test]
    fn default() {
        let expected = TargetAsset {
            symbol: AssetSymbol::default(),
            percent: 0.0,
        };
        assert_eq!(expected, Default::default())
    }
}