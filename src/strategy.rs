use crate::asset::AssetSymbol;
use crate::config::strategy::StrategyConfig;
use crate::error::GreedError;
use crate::platform::quote::Quote;
use crate::platform::FinancialPlatform;
use itertools::Itertools;
use log::info;
use crate::platform::account::Account;

mod symbols;

pub async fn run(
    config: &StrategyConfig,
    platform: &Box<dyn FinancialPlatform>,
) -> Result<(), GreedError> {
    info!("🧠 running strategy: {}", config.name);
    let _ = fetch_account(platform).await?;
    let symbols = symbols::from_config(config);
    let _ = fetch_quotes(symbols, platform).await?;
    info!("----------");
    Ok(())
}

async fn fetch_account(platform: &Box<dyn FinancialPlatform>) -> Result<Account, GreedError> {
    info!("- fetching account info");
    let account = platform.account().await?;
    info!("-- {}", account);
    Ok(account)
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