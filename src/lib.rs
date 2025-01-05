use crate::analysis::AssetAnalyzer;
use crate::asset::AssetSymbol;
use crate::config::platform::PlatformType;
use crate::error::GreedError;
use crate::platform::args::PlatformArgs;
use crate::run::{GreedRunner, GreedRunnerArgs};

mod analysis;
mod assert;
pub mod asset;
mod bool;
pub mod config;
mod date;
mod enum_display;
pub mod error;
mod fixture;
mod float;
mod num;
mod pager;
pub mod platform;
pub mod run;
mod statistics;
mod strategy;
mod tactic;
mod trading_days;

pub async fn greed_loop(args: GreedRunnerArgs) -> Result<(), GreedError> {
    let runner = GreedRunner::from_args(args).await?;
    runner.run_loop().await;
    Ok(())
}

pub async fn fetch_quote(
    assets: &Vec<AssetSymbol>,
    platform_type: &PlatformType,
    platform_args: PlatformArgs,
) -> Result<(), GreedError> {
    print_price_explainer();
    let platform = platform::for_type(platform_type, platform_args)?;
    let quotes = platform.latest_quotes(assets).await?;
    for quote in quotes {
        println!("{}", quote)
    }
    Ok(())
}

pub async fn fetch_status(
    platform_args: PlatformArgs,
    platform_type: &PlatformType,
) -> Result<(), GreedError> {
    let platform = platform::for_type(platform_type, platform_args)?;
    // Fetch account info
    let account = platform.account().await?;
    println!("Account Info: {}", account);
    // Fetch Open Positions
    println!("Open Positions:");
    let positions = platform.positions().await?;
    positions
        .iter()
        .filter(|p| p.market_value.unwrap_or(0.0).abs() >= 0.01)
        .for_each(|p| println!("-- {}", p));
    // Fetch Open Orders
    println!("Open Orders:");
    let orders = platform.open_orders().await?;
    for order in &orders {
        println!("-- {}", order);
    }
    Ok(())
}

fn print_price_explainer() {
    println!("ask price = lowest price where someone is willing to sell a share");
    println!("bid price = highest price someone is willing to pay for a share");
    println!("-----------");
}

pub async fn analyze_stocks(
    assets: &Vec<AssetSymbol>,
    platform_type: &PlatformType,
    platform_args: PlatformArgs,
) -> Result<(), GreedError> {
    let platform = platform::for_type(platform_type, platform_args)?;
    let analyzer = AssetAnalyzer::new(platform);
    let bars_by_symbol = analyzer.analyze_bars(assets).await?;
    for symbol in assets {
        let bars = bars_by_symbol
            .get(symbol)
            .ok_or(GreedError::new("missing bars"))?;
        println!("{}", bars)
    }
    Ok(())
}
