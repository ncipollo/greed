use std::time::Duration;
use tokio::time::sleep;

mod asset;

pub async fn fetch_quote() {
    println!("fetch quote!!")
}

pub async fn greed_loop() {
    loop {
        println!(" 💵");
        sleep(Duration::from_millis(1_000)).await;
    }
}
