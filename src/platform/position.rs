use crate::asset::AssetSymbol;
use crate::platform::asset_class::AssetClass;
use crate::platform::id::Id;
use crate::platform::side::Side;
use num_decimal::Num;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Position {
    pub id: Id,
    /// Asset class of the position
    pub asset_class: AssetClass,
    /// Average entry price of the position.
    pub average_entry_price: Num,
    /// The percent change today.
    pub change_today_percent: Option<Num>,
    /// The total cost basis in dollars.
    pub cost_basis: Num,
    /// The current price per share.
    pub current_price: Option<Num>,
    /// The last day's price per share.
    pub last_day_price: Option<Num>,
    /// The total dollar amount.
    pub market_value: Option<Num>,
    /// The total number of shares in the position.
    pub quantity: Num,
    /// The number of shares minus the amount covering open orders.
    pub quantity_available: Num,
    /// Long or short side.
    pub side: Side,
    /// The symbol associated with the position.
    pub symbol: AssetSymbol,
    /// Today's unrealized profit/loss in dollars.
    pub unrealized_gain_today: Option<Num>,
    /// Today's unrealized profit/loss percent
    pub unrealized_gain_today_percent: Option<Num>,
    /// The total unrealized profit/loss in dollars.
    pub unrealized_gain_total: Option<Num>,
    /// The total unrealized profit/loss percent
    pub unrealized_gain_total_percent: Option<Num>,
}

trait NumFormatter {
    fn format_as_float(&self) -> String;
}

impl Position {
    #[cfg(test)]
    pub fn fixture(symbol: AssetSymbol) -> Self {
        Self {
            quantity: Num::from(50),
            symbol,
            ..Default::default()
        }
    }
}

impl NumFormatter for Option<Num> {
    fn format_as_float(&self) -> String {
        self.as_ref()
            .and_then(|v| v.to_f64())
            .map(|v| format!("{:.2}", v))
            .unwrap_or("--".to_string())
    }
}

impl NumFormatter for Num {
    fn format_as_float(&self) -> String {
        self.to_f64()
            .map(|v| format!("{:.2}", v))
            .unwrap_or("--".to_string())
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let quantity = self.quantity.format_as_float();
        let market_value = self.market_value.format_as_float();
        let unrealized_today = self.unrealized_gain_today_percent.format_as_float();
        let unrealized_total = self.unrealized_gain_total_percent.format_as_float();

        write!(
            f,
            "{} quantity: {}, value: {}, gains today: {}%, gains total: {}%",
            self.symbol, quantity, market_value, unrealized_today, unrealized_total
        )
    }
}

#[cfg(test)]
mod test {
    use crate::asset::AssetSymbol;
    use crate::platform::position::Position;
    use num_decimal::Num;
    use std::str::FromStr;

    #[test]
    fn display_all_optionals_present() {
        let position = Position {
            quantity: Num::from_str("10.1").unwrap(),
            market_value: Num::from_str("100.50").ok(),
            symbol: AssetSymbol::new("vti"),
            unrealized_gain_today_percent: Num::from_str("-5.0").ok(),
            unrealized_gain_total_percent: Num::from_str("10.0").ok(),
            ..Default::default()
        };
        let display = position.to_string();
        let expected =
            "VTI quantity: 10.10, value: 100.50, gains today: -5.00%, gains total: 10.00%";
        assert_eq!(display, expected)
    }

    #[test]
    fn display_no_optionals_present() {
        let position = Position {
            quantity: Num::from_str("10.1").unwrap(),
            symbol: AssetSymbol::new("vti"),
            ..Default::default()
        };
        let display = position.to_string();
        let expected = "VTI quantity: 10.10, value: --, gains today: --%, gains total: --%";
        assert_eq!(display, expected)
    }
}
