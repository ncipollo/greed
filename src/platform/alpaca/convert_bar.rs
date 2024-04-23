pub mod convert_bar_request;
mod convert_time_frame;

use apca::RequestError;
use crate::greed_error_from;
use crate::platform::bar::Bar;
use crate::platform::bars::Bars;

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

greed_error_from!(RequestError<apca::data::v2::bars::ListError>);