use apca::RequestError;

use crate::asset::AssetSymbol;
use crate::greed_error_from;
use crate::platform::bars::Bars;
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

impl From<Bars> for Quote {
    fn from(value: Bars) -> Self {
        let symbol = value.symbol;
        let bar = value.bars.first().cloned().unwrap_or_default();
        Self {
            time: bar.timestamp,
            ask_price: bar.close.clone(),
            ask_size: 0,
            bid_price: bar.close.clone(),
            bid_size: 0,
            symbol,
        }
    }
}

greed_error_from!(RequestError<apca::data::v2::last_quotes::GetError>);

#[cfg(test)]
mod tests {
    use num_decimal::Num;
    use crate::date::DateTimeFixture;
    use super::*;

    #[test]
    fn from_bars() {
        let bars = Bars::fixture(AssetSymbol::new("AAPL"), 100);
        let quote = Quote::from(bars);
        let expected = Quote {
            time: DateTimeFixture::utc(),
            ask_price: Num::from(200),
            bid_price: Num::from(200),
            symbol: AssetSymbol::new("AAPL"),
            ..Default::default()
        };
        assert_eq!(expected, quote);
    }
}