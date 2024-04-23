use apca::data::v2::bars::Adjustment::Split;
use apca::data::v2::bars::ListReq;
use apca::data::v2::Feed;
use crate::platform::bar::bar_request::BarRequest;

impl From<BarRequest> for ListReq {
    fn from(value: BarRequest) -> Self {
        Self {
            symbol: value.symbol.to_string(),
            limit: value.limit,
            start: value.start,
            end: value.end,
            timeframe: value.timeframe.into(),
            adjustment: Some(Split),
            feed: Some(Feed::IEX),
            page_token: None,
            _non_exhaustive: (),
        }
    }
}

#[cfg(test)]
mod tests {
    use apca::data::v2::Feed::IEX;
    use chrono::{TimeZone, Utc};
    use crate::asset::AssetSymbol;
    use crate::platform::bar::time_frame::TimeFrame;
    use super::*;

    #[test]
    fn into_alpaca() {
        let start_time = Utc.timestamp_nanos(0);
        let end_time = Utc.timestamp_nanos(1000);
        let request = BarRequest {
            symbol: AssetSymbol::new("VTI"),
            limit: Some(100),
            start: start_time,
            end: end_time,
            timeframe: TimeFrame::OneDay,
        };
        let alpaca_request: ListReq = request.into();
        let expected = ListReq {
            symbol: "VTI".to_string(),
            limit: Some(100),
            start: start_time,
            end: end_time,
            timeframe: TimeFrame::OneDay.into(),
            adjustment: Some(Split),
            feed: Some(IEX),
            page_token: None,
            _non_exhaustive: (),
        };
        assert_eq!(alpaca_request, expected)
    }
}