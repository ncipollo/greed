use crate::asset::AssetSymbol;
use crate::config::strategy::StrategyConfig;
use crate::error::GreedError;
use crate::platform::quote::Quote;
use crate::platform::FinancialPlatform;
use itertools::Itertools;
use log::info;

mod symbols;

pub async fn run(
    config: &StrategyConfig,
    platform: &Box<dyn FinancialPlatform>,
) -> Result<(), GreedError> {
    info!("🧠 running strategy: {}", config.name);
    let symbols = symbols::from_config(config);
    let _ = fetch_quotes(symbols, platform).await?;
    info!("----------");
    Ok(())
}

async fn fetch_quotes(
    symbols: Vec<AssetSymbol>,
    platform: &Box<dyn FinancialPlatform>,
) -> Result<Vec<Quote>, GreedError> {
    let symbols_string = symbols_string(&symbols);
    info!("- fetching quotes for {}", symbols_string);
    let quotes = platform.latest_quotes(&symbols).await?;
    for quote in &quotes {
        info!("-- {}", quote);
    }
    Ok(quotes)
}

fn symbols_string(symbols: &Vec<AssetSymbol>) -> String {
    symbols.iter().map(|s| s.symbol.clone()).join(",")
}