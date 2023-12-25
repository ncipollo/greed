use crate::analysis::result::BarsResult;
use crate::analysis::time_range::FetcherTimeRanges;
use crate::asset::AssetSymbol;
use crate::error::GreedError;
use crate::platform::bar::bar_request::BarRequest;
use crate::platform::bar::time_frame::TimeFrame;
use crate::platform::bars::Bars;
use crate::platform::FinancialPlatform;
use std::sync::Arc;

pub struct BarsFetcher {
    platform: Arc<dyn FinancialPlatform>,
    time_ranges: FetcherTimeRanges,
}

impl BarsFetcher {
    pub fn new(platform: Arc<dyn FinancialPlatform>) -> Self {
        BarsFetcher {
            platform,
            time_ranges: FetcherTimeRanges::new(),
        }
    }

    pub async fn fetch(&self, symbol: &AssetSymbol) -> Result<BarsResult, GreedError> {
        let bars_result = BarsResult {
            symbol: symbol.clone(),
            yesterday: self.fetch_yesterday(symbol.clone()).await?,
            seven_day: self.fetch_seven_day(symbol.clone()).await?,
            thirty_day: self.fetch_thirty_day(symbol.clone()).await?,
        };
        Ok(bars_result)
    }

    async fn fetch_yesterday(&self, symbol: AssetSymbol) -> Result<Bars, GreedError> {
        let time_range = self.time_ranges.yesterday_range();
        self.platform
            .bars(BarRequest {
                symbol,
                start: time_range.start,
                end: time_range.end,
                timeframe: TimeFrame::OneMinute,
                ..Default::default()
            })
            .await
    }

    async fn fetch_seven_day(&self, symbol: AssetSymbol) -> Result<Bars, GreedError> {
        let time_range = self.time_ranges.last_x_days(7);
        self.platform
            .bars(BarRequest {
                symbol,
                start: time_range.start,
                end: time_range.end,
                timeframe: TimeFrame::OneHour,
                ..Default::default()
            })
            .await
    }

    async fn fetch_thirty_day(&self, symbol: AssetSymbol) -> Result<Bars, GreedError> {
        let time_range = self.time_ranges.last_x_days(30);
        self.platform
            .bars(BarRequest {
                symbol,
                start: time_range.start,
                end: time_range.end,
                timeframe: TimeFrame::OneDay,
                ..Default::default()
            })
            .await
    }
}
