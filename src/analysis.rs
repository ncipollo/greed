use crate::analysis::fetcher::BarsFetcher;
use crate::analysis::result::BarsResult;
use crate::analysis::state::AnalysisState;
use crate::asset::AssetSymbol;
use crate::clock::{Clock, SystemClock};
use crate::error::GreedError;
use crate::platform::FinancialPlatform;
use chrono::Local;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub mod fetcher;
pub mod result;
mod state;
pub(crate) mod time_range;

pub struct AssetAnalyzer {
    mutable_state: Arc<Mutex<AnalysisState>>,
    platform: Arc<dyn FinancialPlatform>,
    clock: Arc<dyn Clock>,
}

impl AssetAnalyzer {
    pub fn new(platform: Arc<dyn FinancialPlatform>) -> Self {
        Self::with_clock(platform, Arc::new(SystemClock))
    }

    /// Bars are refetched once per calendar day as seen by `clock`.
    pub fn with_clock(platform: Arc<dyn FinancialPlatform>, clock: Arc<dyn Clock>) -> Self {
        Self {
            mutable_state: Arc::new(Mutex::new(Default::default())),
            platform,
            clock,
        }
    }

    pub async fn analyze_bars(
        &self,
        assets: &Vec<AssetSymbol>,
    ) -> Result<Arc<HashMap<AssetSymbol, BarsResult>>, GreedError> {
        self.refresh_bars_if_needed(assets).await?;
        Ok(self.bars_by_symbol())
    }

    async fn refresh_bars_if_needed(&self, assets: &Vec<AssetSymbol>) -> Result<(), GreedError> {
        let now = self.clock.now().with_timezone(&Local);
        let should_fetch = self.mutable_state.lock().unwrap().should_fetch(now);
        if should_fetch {
            let fetch_result = self.fetch_bars(assets).await?;
            *self.mutable_state.lock().unwrap() = AnalysisState::new(fetch_result, now);
        }
        Ok(())
    }

    async fn fetch_bars(
        &self,
        assets: &Vec<AssetSymbol>,
    ) -> Result<HashMap<AssetSymbol, BarsResult>, GreedError> {
        let fetcher = BarsFetcher::with_clock(self.platform.clone(), self.clock.as_ref());
        let mut results_by_symbol = HashMap::new();
        for symbol in assets {
            let results = fetcher.fetch(symbol).await?;
            results_by_symbol.insert(symbol.clone(), results);
        }
        Ok(results_by_symbol)
    }

    fn bars_by_symbol(&self) -> Arc<HashMap<AssetSymbol, BarsResult>> {
        self.mutable_state.lock().unwrap().bars_by_symbol.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::clock::SimulatedClock;
    use crate::platform::MockPlatform;
    use chrono::{Duration, TimeZone, Utc};

    #[tokio::test]
    async fn refetches_bars_when_the_simulated_clock_crosses_midnight() {
        let platform = Arc::new(MockPlatform::new());
        let clock = Arc::new(SimulatedClock::new(
            Utc.with_ymd_and_hms(2026, 1, 5, 12, 0, 0).unwrap(),
        ));
        let analyzer = AssetAnalyzer::with_clock(platform.clone(), clock.clone());
        let assets = vec![AssetSymbol::new("VTI")];

        analyzer.analyze_bars(&assets).await.unwrap();
        let after_first = platform.bar_requests().len();
        analyzer.analyze_bars(&assets).await.unwrap();
        let after_same_day = platform.bar_requests().len();
        clock.advance(Duration::hours(26));
        analyzer.analyze_bars(&assets).await.unwrap();

        assert!(after_first > 0);
        assert_eq!(after_same_day, after_first);
        assert_eq!(platform.bar_requests().len(), after_first * 2);
    }

    #[tokio::test]
    async fn bar_windows_are_measured_from_the_clock() {
        let platform = Arc::new(MockPlatform::new());
        let clock = Arc::new(SimulatedClock::new(
            Utc.with_ymd_and_hms(2024, 4, 22, 12, 0, 0).unwrap(),
        ));
        let analyzer = AssetAnalyzer::with_clock(platform.clone(), clock);

        analyzer
            .analyze_bars(&vec![AssetSymbol::new("VTI")])
            .await
            .unwrap();

        // Monday 2024-04-22: the last trading day is Friday 2024-04-19.
        let first = &platform.bar_requests()[0];
        assert_eq!(
            first.start,
            Utc.with_ymd_and_hms(2024, 4, 19, 0, 0, 0).unwrap()
        );
    }
}
