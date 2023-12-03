use crate::asset::AssetSymbol;
use crate::platform::bar::Bar;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Bars {
    pub symbol: AssetSymbol,
    pub bars: Vec<Bar>,
}