use crate::greed_error_from;
use crate::platform::account::Account;
use crate::platform::id::Id;
use apca::RequestError;

impl From<apca::api::v2::account::Account> for Account {
    fn from(value: apca::api::v2::account::Account) -> Self {
        Self {
            id: Id::Uuid(value.id.0),
            buying_power: value.buying_power,
            cash: value.cash,
            currency: value.currency.to_string(),
            daytrade_count: value.daytrade_count,
            day_trader: value.day_trader,
            market_value_long: value.market_value_long,
            market_value_short: value.market_value_short,
            equity: value.equity,
        }
    }
}

greed_error_from!(RequestError<apca::api::v2::account::GetError>);
