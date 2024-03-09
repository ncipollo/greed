use crate::asset::AssetSymbol;

#[derive(Debug, PartialEq)]
pub struct TargetAsset {
    pub symbol: AssetSymbol,
    pub percent: f64,
}

impl Default for TargetAsset {
    fn default() -> Self {
        Self {
            symbol: AssetSymbol::default(),
            percent: 0.0,
        }
    }
}

impl TargetAsset {
    pub fn full_percent(symbol: AssetSymbol) -> Self {
        Self {
            symbol,
            percent: 100.0,
        }
    }

    #[cfg(test)]
    pub fn fixture() -> Self {
        Self::full_percent(AssetSymbol::new("SPY"))
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

    #[test]
    fn full_percent() {
        let target = TargetAsset::full_percent(AssetSymbol::new("SPY"));
        let expected = TargetAsset {
            symbol: AssetSymbol::new("SPY"),
            percent: 100.0,
        };
        assert_eq!(expected, target)
    }
}
