mod cli;

use crate::cli::{Cli, Command};
use apca::api::v2::orders::{OrdersReq, Status};
use apca::api::v2::{account, orders, positions};
use apca::data::v2::last_quotes::LastQuotesReqInit;
use apca::data::v2::quotes::QuotesReqInit;
use apca::data::v2::{last_quotes, quotes};
use apca::{ApiInfo, Client, RequestError};
use chrono::{Duration, Utc};
use clap::{Parser};
use greed::{fetch_quote, greed_loop};
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let command = cli.command;
    match command {
        Command::Run => {
            setup_logging();
            greed_loop().await;
        }
        Command::Quote(_debug_options) => fetch_quote().await,
        Command::TestAlpaca => test_alpaca().await,
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
        println!("VTI Historic Quote: {:?}", quote);
    }

    println!("Latest VTI: ---");
    let latest_req = LastQuotesReqInit {
        ..Default::default()
    }
    .init(vec!["VTI"]);

    let latest = client.issue::<last_quotes::Get>(&latest_req).await.unwrap();
    let quote = latest[0].clone().1;
    let ask = quote.ask_price.to_f64().unwrap();
    let bid = quote.bid_price.to_f64().unwrap();
    println!("VTI Ask and bid: {ask} , {bid}");

    // Uncomment 👇 to test buy order
    // let buy_order_req = order::OrderReqInit {
    //     type_: Type::Market,
    //     ..Default::default()
    // }
    // .init("VTI", Side::Buy, order::Amount::notional(10));
    //
    // let buy_order = client.issue::<order::Post>(&buy_order_req).await.unwrap();
    // println!("Order status: {:?}", buy_order.status);

    let orders_req = OrdersReq {
        status: Status::All,
        ..OrdersReq::default()
    };
    let orders = client.issue::<orders::Get>(&orders_req).await.unwrap();

    println!("Orders: ---");
    orders.iter().for_each(|order| {
        println!(
            "Order - {}, amount - {:?}, status, {:?}",
            order.symbol, order.amount, order.status
        )
    });

    println!("Positions: ---");
    let positions = client.issue::<positions::Get>(&()).await.unwrap();
    positions.iter().for_each(|position| {
        println!(
            "Position - {}, amount - {:?}, unrealized gain percent - {:?}",
            position.symbol,
            position.quantity.to_f64(),
            position.unrealized_gain_total_percent.clone().unwrap().to_f64()
        )
    });
}

fn setup_logging() {
    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .expect("Failed to initialize logger")
}
