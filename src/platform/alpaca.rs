mod convert_account;
mod convert_asset_class;
mod convert_position;
mod convert_quote;
mod convert_side;
mod factory;

use crate::asset::AssetSymbol;
use crate::error::GreedError;
use crate::platform::account::Account;
use crate::platform::alpaca::factory::create_alpaca_client;
use crate::platform::args::PlatformArgs;
use crate::platform::quote::Quote;
use crate::platform::FinancialPlatform;
use apca::api::v2::{account, positions};
use apca::data::v2::last_quotes;
use apca::data::v2::last_quotes::LastQuotesReqInit;
use apca::Client;
use async_trait::async_trait;
use itertools::Itertools;
use crate::platform::position::Position;

pub struct AlpacaPlatform {
    client: Client,
}

impl AlpacaPlatform {
    pub fn new(runner_args: &PlatformArgs) -> Result<Self, GreedError> {
        Ok(Self {
            client: create_alpaca_client(runner_args.is_simulated)?,
        })
    }
}

#[async_trait]
impl FinancialPlatform for AlpacaPlatform {
    async fn account(&self) -> Result<Account, GreedError> {
        let alpaca_account = self.client.issue::<account::Get>(&()).await?;
        Ok(alpaca_account.into())
    }

    async fn latest_quotes(&self, symbols: &Vec<AssetSymbol>) -> Result<Vec<Quote>, GreedError> {
        let symbol_strings = symbols.iter().map(|s| &s.symbol).collect::<Vec<_>>();
        let latest_req = LastQuotesReqInit {
            ..Default::default()
        }
        .init(symbol_strings);

        let result = self.client.issue::<last_quotes::Get>(&latest_req).await?;
        let quotes: Vec<Quote> = result.into_iter().map(|q| q.into()).collect();
        Ok(quotes)
    }

    async fn positions(&self) -> Result<Vec<Position>, GreedError> {
        let alpaca_positions = self.client.issue::<positions::Get>(&()).await?;
        let positions: Vec<Position> = alpaca_positions.into_iter().map_into().collect();
        Ok(positions)
    }
}
