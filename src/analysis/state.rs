use crate::analysis::result::BarsResult;
use crate::asset::AssetSymbol;
use chrono::{DateTime, Local};
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Default, PartialEq)]
pub struct AnalysisState {
    pub bars_by_symbol: Rc<HashMap<AssetSymbol, BarsResult>>,
    last_fetch_time: DateTime<Local>,
}

impl AnalysisState {
    pub fn new(
        bars_by_symbol: HashMap<AssetSymbol, BarsResult>,
        last_fetch_time: DateTime<Local>,
    ) -> Self {
        Self {
            bars_by_symbol: Rc::new(bars_by_symbol),
            last_fetch_time,
        }
    }
    pub fn should_fetch(&self, now: DateTime<Local>) -> bool {
        let now_date = now.date_naive();
        let last_fetch_date = self.last_fetch_time.date_naive();

        now_date != last_fetch_date
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::date::DateTimeFixture;
    use chrono::Duration;

    #[test]
    fn should_fetch_dates_are_same() {
        let last_fetch = DateTimeFixture::local();
        let state = AnalysisState::new(HashMap::new(), last_fetch);
        let now = last_fetch + Duration::minutes(5);

        assert!(!state.should_fetch(now))
    }

    #[test]
    fn should_fetch_dates_are_different() {
        let last_fetch = DateTimeFixture::local();
        let state = AnalysisState::new(HashMap::new(), last_fetch);
        let now = last_fetch + Duration::days(1);

        assert!(state.should_fetch(now))
    }
}
