use crate::asset::AssetSymbol;
use crate::config::platform::PlatformType;
use crate::error::GreedError;
use crate::platform::args::PlatformArgs;
use crate::run::{GreedRunner, GreedRunnerArgs};

pub mod asset;
pub mod config;
pub mod error;
mod fixture;
pub mod platform;
pub mod run;
mod strategy;
mod enum_display;

pub async fn greed_loop(args: GreedRunnerArgs) -> Result<(), GreedError> {
    let runner = GreedRunner::from_args(args).await?;
    runner.run_loop().await?;
    Ok(())
}

pub async fn fetch_quote(assets: &Vec<AssetSymbol>,
                         platform_type: &PlatformType,
                         platform_args: PlatformArgs) -> Result<(), GreedError> {
    print_price_explainer();
    let platform = platform::for_type(platform_type, platform_args)?;
    let quotes = platform.latest_quotes(assets).await?;
    for quote in quotes {
        println!("{}", quote)
    }
    Ok(())
}

fn print_price_explainer() {
    println!("ask price = lowest price where someone is willing to sell a share");
    println!("bid price = highest price someone is willing to pay for a share");
    println!("-----------");
}