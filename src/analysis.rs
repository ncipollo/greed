use crate::analysis::fetcher::BarsFetcher;
use crate::analysis::result::BarsResult;
use crate::asset::AssetSymbol;
use crate::error::GreedError;
use crate::platform::FinancialPlatform;
use std::collections::HashMap;
use std::sync::Arc;

pub mod fetcher;
pub mod result;
mod time_range;

pub async fn analyze_bars(
    platform: Arc<dyn FinancialPlatform>,
    assets: &Vec<AssetSymbol>,
) -> Result<HashMap<AssetSymbol, BarsResult>, GreedError> {
    let fetcher = BarsFetcher::new(platform.clone());
    let mut results_by_symbol = HashMap::new();
    for symbol in assets {
        let results = fetcher.fetch(symbol).await?;
        results_by_symbol.insert(symbol.clone(), results);
    }
    Ok(results_by_symbol)
}
