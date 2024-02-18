use crate::asset::AssetSymbol;

pub trait ForRule {

}

#[derive(Debug, Default, PartialEq)]
struct ForResult {
    target_assets: Vec<TargetAsset>
}

#[derive(Debug, PartialEq)]
struct TargetAsset {
    symbol: AssetSymbol,
    percent: f64
}

impl Default for TargetAsset {
    fn default() -> Self {
        Self {
            symbol: AssetSymbol::default(),
            percent: 1.0
        }
    }
}