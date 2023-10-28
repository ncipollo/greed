use std::time::Duration;
use log::info;
use tokio::time::sleep;

mod asset;
mod config;
mod error;
mod fixture;

pub async fn fetch_quote() {
    println!("fetch quote!!")
}

pub async fn greed_loop() {
    loop {
        info!(" 💵");
        sleep(Duration::from_millis(1_000)).await;
    }
}
