use chrono::{DateTime, Utc};
use crate::asset::AssetSymbol;
use crate::platform::bar::time_frame::TimeFrame;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BarRequest {
    pub symbol: AssetSymbol,
    pub limit: Option<usize>,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub timeframe: TimeFrame
}