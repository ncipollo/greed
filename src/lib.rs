use crate::analysis::AssetAnalyzer;
use crate::asset::AssetSymbol;
use crate::config::platform::PlatformType;
use crate::error::GreedError;
use crate::interval::RollupInterval;
use crate::output_format::OutputFormat;
use crate::platform::args::PlatformArgs;
use crate::platform::quote::Quote;
use crate::run::{GreedRunner, GreedRunnerArgs};
use chrono::{DateTime, Utc};
use comfy_table::Table;
use std::ops::Range;
use std::path::PathBuf;

mod analysis;
mod assert;
pub mod asset;
mod bool;
pub mod config;
mod date;
mod db;
mod enum_display;
pub mod error;
mod fixture;
mod float;
pub mod history_range;
pub mod interval;
mod num;
pub mod output_format;
mod pager;
pub mod platform;
mod quote_accumulator;
pub mod run;
mod statistics;
mod strategy;
mod tactic;
pub mod template;
mod trading_days;

pub async fn greed_loop(args: GreedRunnerArgs) -> Result<(), GreedError> {
    let db_path = db::path_for_run(&args.config_path, args.db_path.as_deref());
    db::bootstrap_at_with_retry(&db_path).await?;
    let runner = GreedRunner::from_args(args, db_path).await?;
    runner.run_loop().await;
    Ok(())
}

pub async fn fetch_quote(
    assets: &[AssetSymbol],
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

pub async fn fetch_recent_orders(
    platform_args: PlatformArgs,
    platform_type: &PlatformType,
) -> Result<(), GreedError> {
    let platform = platform::for_type(platform_type, platform_args)?;
    let orders = platform.recent_orders().await?;
    println!("Recent Orders:");
    for order in orders {
        println!("-- {}", order.display_with_time_zone(&chrono::Local))
    }
    Ok(())
}

pub async fn fetch_status(
    platform_args: PlatformArgs,
    platform_type: &PlatformType,
    show_full_status: bool,
) -> Result<(), GreedError> {
    let platform = platform::for_type(platform_type, platform_args)?;
    // Fetch account info
    let account = platform.account().await?;
    println!("Account Info: {}", account);
    // Bail if we're not showing full status
    if !show_full_status {
        return Ok(());
    }
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

pub async fn fetch_portfolio_status_history(
    range: Range<DateTime<Utc>>,
    interval: Option<RollupInterval>,
    format: OutputFormat,
    db_path: Option<PathBuf>,
) -> Result<(), GreedError> {
    let path = db::path_for_override(db_path.as_deref());
    match interval {
        Some(interval) => {
            let rollups = match db::open_read_only_with_retry(&path).await? {
                Some(connection) => {
                    db::portfolio_status::select_rollup_in_range(&connection, range, &interval)?
                }
                None => Vec::new(),
            };
            match format {
                OutputFormat::Text => {
                    for rollup in rollups {
                        println!("{}", rollup);
                    }
                }
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&rollups)?);
                }
            }
        }
        None => {
            let records = match db::open_read_only_with_retry(&path).await? {
                Some(connection) => db::portfolio_status::select_in_range(&connection, range)?,
                None => Vec::new(),
            };
            match format {
                OutputFormat::Text => {
                    for record in records {
                        println!("{}", record);
                    }
                }
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&records)?);
                }
            }
        }
    }
    Ok(())
}

pub async fn fetch_stock_quote_history(
    symbols: &[AssetSymbol],
    range: Range<DateTime<Utc>>,
    interval: Option<RollupInterval>,
    format: OutputFormat,
    db_path: Option<PathBuf>,
) -> Result<(), GreedError> {
    let path = db::path_for_override(db_path.as_deref());
    match interval {
        Some(interval) => {
            let rollups = match db::open_read_only_with_retry(&path).await? {
                Some(connection) => {
                    db::stock_quotes::select_rollup_in_range(&connection, range, &interval)?
                        .into_iter()
                        .filter(|r| matches_symbol_filter(symbols, &r.symbol))
                        .collect()
                }
                None => Vec::new(),
            };
            match format {
                OutputFormat::Text => {
                    let mut table = Table::new();
                    table.set_header(["Bucket", "Symbol", "Avg Ask", "Avg Bid"]);
                    for rollup in rollups {
                        table.add_row([
                            rollup.bucket.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                            rollup.symbol.to_string(),
                            format!("{:.2}", rollup.avg_ask_price),
                            format!("{:.2}", rollup.avg_bid_price),
                        ]);
                    }
                    println!("{table}");
                }
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&rollups)?);
                }
            }
        }
        None => {
            let quotes: Vec<Quote> = match db::open_read_only_with_retry(&path).await? {
                Some(connection) => db::stock_quotes::select_in_range(&connection, range)?
                    .into_iter()
                    .filter(|q| matches_symbol_filter(symbols, &q.symbol))
                    .collect(),
                None => Vec::new(),
            };
            match format {
                OutputFormat::Text => {
                    let mut table = Table::new();
                    table.set_header(["Time", "Symbol", "Ask", "Bid", "Spread", "Spread %"]);
                    for quote in quotes {
                        table.add_row([
                            quote.fmt_time(&chrono::Local),
                            quote.symbol.to_string(),
                            format!("{:.2}", quote.ask_price),
                            format!("{:.2}", quote.bid_price),
                            format!("{:.2}", quote.spread()),
                            format!("{:.2}%", quote.spread_percent()),
                        ]);
                    }
                    println!("{table}");
                }
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&quotes)?);
                }
            }
        }
    }
    Ok(())
}

fn matches_symbol_filter(symbols: &[AssetSymbol], symbol: &AssetSymbol) -> bool {
    symbols.is_empty() || symbols.contains(symbol)
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn matches_symbol_filter_empty_list_matches_everything() {
        assert!(matches_symbol_filter(&[], &AssetSymbol::new("VTI")));
    }

    #[test]
    fn matches_symbol_filter_matches_listed_symbol() {
        let symbols = vec![AssetSymbol::new("VTI"), AssetSymbol::new("AAPL")];
        assert!(matches_symbol_filter(&symbols, &AssetSymbol::new("VTI")));
    }

    #[test]
    fn matches_symbol_filter_rejects_unlisted_symbol() {
        let symbols = vec![AssetSymbol::new("AAPL")];
        assert!(!matches_symbol_filter(&symbols, &AssetSymbol::new("VTI")));
    }
}
