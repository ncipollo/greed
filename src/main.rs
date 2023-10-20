use apca::{ApiInfo, Client};
use apca::api::v2::account;

#[tokio::main]
async fn main() {
    let api_info = ApiInfo::from_env().expect("failed to load alpaca info from env");
    let client = Client::new(api_info);
    let account = client.issue::<account::Get>(&()).await.unwrap();
    let power = account.buying_power;
    println!("Hello buying power: {power}");
}
