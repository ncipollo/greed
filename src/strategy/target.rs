use std::ops::Mul;

use crate::asset::AssetSymbol;

#[derive(Clone, Debug, PartialEq)]
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
    #[cfg(test)]
    pub  fn new(symbol: AssetSymbol, percent: f64) -> Self {
        Self { symbol, percent }
    }

    pub fn full_percent(symbol: AssetSymbol) -> Self {
        Self {
            symbol,
            percent: 100.0,
        }
    }

    pub fn apply_percent(&self, to_num: f64) -> f64 {
        let percent = self.percent / 100.0;
        to_num * percent
    }

    #[cfg(test)]
    pub fn fixture() -> Self {
        Self::full_percent(AssetSymbol::new("SPY"))
    }
}

impl Mul<f64> for TargetAsset {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            symbol: self.symbol,
            percent: self.percent * rhs,
        }
    }
}

#[cfg(test)]
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
    fn multiply_f64() {
        let target_asset = TargetAsset {
            symbol: AssetSymbol::new("SPY"),
            percent: 50.0,
        };
        let result = target_asset * 0.5;
        let expected = TargetAsset {
            symbol: AssetSymbol::new("SPY"),
            percent: 25.0,
        };
        assert_eq!(expected, result)
    }

    #[test]
    fn apply_percent() {
        let target_asset = TargetAsset {
            symbol: AssetSymbol::new("SPY"),
            percent: 50.0,
        };
        let result = target_asset.apply_percent(100.0);
        assert_eq!(50.0, result)
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
