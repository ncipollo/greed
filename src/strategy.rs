use crate::asset::AssetSymbol;
use crate::config::strategy::StrategyConfig;
use crate::error::GreedError;
use crate::platform::account::Account;
use crate::platform::position::Position;
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
    let _ = fetch_account(platform).await?;
    let symbols = symbols::from_config(config);
    let _ = fetch_quotes(symbols, platform).await?;
    let _ = fetch_positions(platform).await?;
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

async fn fetch_positions(
    platform: &Box<dyn FinancialPlatform>,
) -> Result<Vec<Position>, GreedError> {
    info!("- fetching open positions");
    let positions = platform.positions().await?;
    for position in &positions {
        info!("-- {}", position);
    }
    Ok(positions)
}

fn symbols_string(symbols: &Vec<AssetSymbol>) -> String {
    symbols.iter().map(|s| s.symbol.clone()).join(",")
}
