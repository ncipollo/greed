mod convert_bar_request;
mod convert_time_frame;

use crate::platform::bar::{Bar, Bars};

impl From<apca::data::v2::bars::Bar> for Bar {
    fn from(value: apca::data::v2::bars::Bar) -> Self {
        Self {
            timestamp: value.time,
            open: value.open,
            close: value.close,
            high: value.high,
            low: value.low,
            volume: value.volume,
        }
    }
}

impl From<apca::data::v2::bars::Bars> for Bars {
    fn from(value: apca::data::v2::bars::Bars) -> Self {
        Self {
            symbol: value.symbol.parse().unwrap(),
            bars: value.bars.into_iter().map(|b| b.into()).collect(),
        }
    }
}
