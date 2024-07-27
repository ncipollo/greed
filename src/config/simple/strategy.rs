use crate::asset::AssetSymbol;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct SimpleStrategyConfig {
    pub asset: AssetSymbol,
    pub buy: Option<f64>,
    pub sell: Option<f64>,
}
