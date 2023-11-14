use serde::{Deserialize, Serialize};
use crate::config::rules::target::PortfolioTargetRule;

mod merge;
pub mod target;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct BuyRulesConfig {
    #[serde(default)]
    pub portfolio_target: Option<PortfolioTargetRule>
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct SellRulesConfig {

}