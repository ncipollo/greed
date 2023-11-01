use std::env;
use crate::error::GreedError;
use apca::{ApiInfo, Client};

pub fn create_alpaca_client(is_simulated: bool) -> Result<Client, GreedError> {
    let api_info = api_info(is_simulated)?;
    Ok(Client::new(api_info))
}

fn api_info(is_simulated: bool) -> Result<ApiInfo, GreedError> {
    let info = if is_simulated {
        let base_url = env::var("SIMULATED_APCA_API_BASE_URL")
            .unwrap_or("https://paper-api.alpaca.markets".to_string());
        let key_id = env::var("SIMULATED_APCA_API_KEY_ID")?;
        let secret_key = env::var("SIMULATED_APCA_API_SECRET_KEY")?;
        ApiInfo::from_parts(base_url, key_id, secret_key)?
    } else {
        ApiInfo::from_env()?
    };
    Ok(info)
}
