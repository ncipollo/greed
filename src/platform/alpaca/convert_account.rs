use crate::greed_error_from;
use crate::platform::account::Account;
use crate::platform::id::Id;
use apca::RequestError;

impl From<apca::api::v2::account::Account> for Account {
    fn from(value: apca::api::v2::account::Account) -> Self {
        Self {
            id: Id::Uuid(value.id.0),
            buying_power: value.buying_power.to_f64().unwrap_or(0.0),
            cash: value.cash.to_f64().unwrap_or(0.0),
            currency: value.currency.to_string(),
            daytrade_count: value.daytrade_count,
            day_trader: value.day_trader,
            market_value_long: value.market_value_long.to_f64().unwrap_or(0.0),
            market_value_short: value.market_value_short.to_f64().unwrap_or(0.0),
            equity: value.equity.to_f64().unwrap_or(0.0),
        }
    }
}

greed_error_from!(RequestError<apca::api::v2::account::GetError>);
