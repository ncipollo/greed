use crate::asset::AssetSymbol;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct SimpleStrategyConfig {
    asset: AssetSymbol,
    buy: Option<f64>,
    sell: Option<f64>,
}
