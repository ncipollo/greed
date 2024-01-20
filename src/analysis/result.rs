use crate::asset::AssetSymbol;
use crate::platform::bars::Bars;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BarsResult {
    pub symbol: AssetSymbol,
    pub yesterday: Bars,
    pub seven_day: Bars,
    pub thirty_day: Bars,
}

impl BarsResult {
    #[cfg(test)]
    pub fn fixture(symbol: AssetSymbol) -> Self {
        Self {
            symbol: symbol.clone(),
            yesterday: Bars::fixture(symbol.clone(), 300),
            seven_day: Bars::fixture(symbol.clone(), 200),
            thirty_day: Bars::fixture(symbol.clone(), 100),
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
            ----------
        ",
            self.symbol,
            self.yesterday.average_median().unwrap_or_default(),
            self.seven_day.average_median().unwrap_or_default(),
            self.thirty_day.average_median().unwrap_or_default(),
        )
    }
}
