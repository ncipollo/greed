mod cli;

use crate::cli::{Cli, Command};
use apca::api::v2::account;
use apca::data::v2::last_quotes::LastQuotesReqInit;
use apca::data::v2::quotes::QuotesReqInit;
use apca::data::v2::{last_quotes, quotes};
use apca::{ApiInfo, Client, RequestError};
use chrono::{Duration, Utc};
use clap::{Parser, Subcommand};
use greed::{fetch_quote, greed_loop};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let command = cli.command;
    match command {
        Command::Run => greed_loop().await,
        Command::Quote(debug_options) => fetch_quote().await,
        Command::TestAlpaca => test_alpaca().await
    }
}

async fn test_alpaca() {
    let api_info = ApiInfo::from_env().expect("failed to load alpaca info from env");
    let client = Client::new(api_info);
    let account = client.issue::<account::Get>(&()).await.unwrap();
    let power = account.buying_power;
    println!("Hello buying power: {power}");

    println!("VTI Historic Quotes: ---");
    let end = Utc::now() - Duration::minutes(15);
    let start = end - Duration::hours(2);
    let quote_req = QuotesReqInit {
        ..Default::default()
    }
    .init("VTI", start, end);
    let quote_result = client.issue::<quotes::Get>(&quote_req).await;
    if quote_result.is_err() {
        let err = quote_result.unwrap_err();
        println!("Endpoint err - {err}");
        if let RequestError::Endpoint(get_err) = err {
            println!("Get err - {get_err}")
        }
    } else {
        let quote = quote_result.unwrap();
        println!("Quote: {:?}", quote);
    }

    println!("Latest VTI: ---");
    let latest_req = LastQuotesReqInit {
        ..Default::default()
    }
    .init(vec!["VTI", "SPY"]);

    let latest = client.issue::<last_quotes::Get>(&latest_req).await.unwrap();
    let quote = latest[0].clone().1;
    let ask = quote.ask_price.to_f64().unwrap();
    let bid = quote.bid_price.to_f64().unwrap();
    println!("Spy and VTI: {:?}", latest);
    println!("Ask and bid: {ask} , {bid}")
}
