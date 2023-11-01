use std::time::Duration;
use log::info;
use tokio::time::sleep;
use crate::error::GreedError;
use crate::run::{GreedRunner, GreedRunnerArgs};

mod asset;
pub mod config;
mod error;
mod fixture;
mod platform;
pub mod run;

pub async fn fetch_quote() {
    println!("fetch quote!!")
}

pub async fn greed_loop(args: GreedRunnerArgs) -> Result<(), GreedError> {
    let runner = GreedRunner::from_args(args).await?;
    runner.run_loop().await;
    Ok(())
}
