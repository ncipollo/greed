use crate::analysis::fetcher::BarsFetcher;
use crate::analysis::result::BarsResult;
use crate::analysis::state::AnalysisState;
use crate::asset::AssetSymbol;
use crate::error::GreedError;
use crate::platform::FinancialPlatform;
use chrono::Local;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

pub mod fetcher;
pub mod result;
mod state;
mod time_range;

pub struct AssetAnalyzer {
    mutable_state: Arc<Mutex<AnalysisState>>,
    platform: Arc<dyn FinancialPlatform>,
}

impl AssetAnalyzer {
    pub fn new(platform: Arc<dyn FinancialPlatform>) -> Self {
        Self {
            mutable_state: Arc::new(Mutex::new(Default::default())),
            platform: platform.clone(),
        }
    }

    pub async fn analyze_bars(
        &self,
        assets: &Vec<AssetSymbol>,
    ) -> Result<Rc<HashMap<AssetSymbol, BarsResult>>, GreedError> {
        self.refresh_bars_if_needed(assets).await?;
        Ok(self.bars_by_symbol())
    }

    async fn refresh_bars_if_needed(
        &self,
        assets: &Vec<AssetSymbol>,
    ) -> Result<(), GreedError> {
        let mutable_state = self.mutable_state.clone();
        let mut state = mutable_state.lock().unwrap();
        if state.should_fetch(Local::now()) {
            let fetch_result = self.fetch_bars(assets).await?;
            *state = AnalysisState::new(fetch_result, Local::now());
        }
        Ok(())
    }

    async fn fetch_bars(
        &self,
        assets: &Vec<AssetSymbol>,
    ) -> Result<HashMap<AssetSymbol, BarsResult>, GreedError> {
        let fetcher = BarsFetcher::new(self.platform.clone());
        let mut results_by_symbol = HashMap::new();
        for symbol in assets {
            let results = fetcher.fetch(symbol).await?;
            results_by_symbol.insert(symbol.clone(), results);
        }
        Ok(results_by_symbol)
    }

    fn bars_by_symbol(&self) -> Rc<HashMap<AssetSymbol, BarsResult>> {
        self.mutable_state.clone().lock().unwrap().bars_by_symbol.clone()
    }
}
