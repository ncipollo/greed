use std::collections::HashMap;
use chrono::{DateTime, Local};
use crate::analysis::result::BarsResult;
use crate::asset::AssetSymbol;

struct AnalysisState {
    bars_by_symbol: HashMap<AssetSymbol, BarsResult>,
    last_fetch_time: DateTime<Local>
}