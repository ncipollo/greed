use apca::RequestError;
use crate::asset::AssetSymbol;
use crate::greed_error_from;
use crate::platform::quote::Quote;

impl From<(String, apca::data::v2::last_quotes::Quote)> for Quote {
    fn from(value: (String, apca::data::v2::last_quotes::Quote)) -> Self {
        Self {
            time: value.1.time,
            ask_price: value.1.ask_price,
            ask_size: value.1.ask_size,
            bid_price: value.1.bid_price,
            bid_size: value.1.bid_size,
            symbol: AssetSymbol::new(&value.0),
        }
    }
}

greed_error_from!(RequestError<apca::data::v2::last_quotes::GetError>);