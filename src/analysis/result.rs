use crate::asset::AssetSymbol;
use crate::platform::bars::Bars;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BarsResult {
    pub symbol: AssetSymbol,
    pub last_trading_day: Bars,
    pub seven_day: Bars,
    pub thirty_day: Bars,
    pub thirty_day_hourly: Bars,
}

impl BarsResult {
    #[cfg(test)]
    pub fn fixture(symbol: AssetSymbol) -> Self {
        Self {
            symbol: symbol.clone(),
            last_trading_day: Bars::fixture(symbol.clone(), 300.0),
            seven_day: Bars::fixture(symbol.clone(), 200.0),
            thirty_day: Bars::fixture(symbol.clone(), 100.0),
            thirty_day_hourly: Bars::fixture(symbol.clone(), 400.0),
        }
    }
}

impl Display for BarsResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\
            Asset: {}\n\
            yesterday median: {:.2}\n\
            7 day median: {:.2}\n\
            30 day median: {:.2}\n\
            Median Positive Change: {:.2}%\n\
            Median Negative Change: {:.2}%\n\
            ----------
        ",
            self.symbol,
            self.last_trading_day.average_median().unwrap_or_default(),
            self.seven_day.average_median().unwrap_or_default(),
            self.thirty_day.average_median().unwrap_or_default(),
            self.thirty_day_hourly.positive_percent_median().unwrap_or_default(),
            self.thirty_day_hourly.negative_percent_median().unwrap_or_default(),
        )
    }
}
