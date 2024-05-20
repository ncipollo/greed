use std::fmt::{Display, Formatter};

use num_decimal::Num;

use crate::asset::AssetSymbol;
use crate::platform::asset_class::AssetClass;
use crate::platform::id::Id;
use crate::platform::side::Side;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Position {
    pub id: Id,
    /// Asset class of the position
    pub asset_class: AssetClass,
    /// Average entry price of the position.
    pub average_entry_price: f64,
    /// The percent change today.
    pub change_today_percent: Option<Num>,
    /// The total cost basis in dollars.
    pub cost_basis: Num,
    /// The current price per share.
    pub current_price: Option<f64>,
    /// The last day's price per share.
    pub last_day_price: Option<f64>,
    /// The total dollar amount.
    pub market_value: Option<f64>,
    /// The total number of shares in the position.
    pub quantity: f64,
    /// The number of shares minus the amount covering open orders.
    pub quantity_available: f64,
    /// Long or short side.
    pub side: Side,
    /// The symbol associated with the position.
    pub symbol: AssetSymbol,
    /// Today's unrealized profit/loss in dollars.
    pub unrealized_gain_today: Option<f64>,
    /// Today's unrealized profit/loss percent
    pub unrealized_gain_today_percent: Option<f64>,
    /// The total unrealized profit/loss in dollars.
    pub unrealized_gain_total: Option<f64>,
    /// The total unrealized profit/loss percent
    pub unrealized_gain_total_percent: Option<f64>,
}

trait NumFormatter {
    fn format_as_float(&self) -> String;
}

impl Position {
    #[cfg(test)]
    pub fn fixture(symbol: AssetSymbol) -> Self {
        Self {
            quantity: 50.0,
            quantity_available: 50.0,
            symbol,
            ..Default::default()
        }
    }
}

impl NumFormatter for Option<f64> {
    fn format_as_float(&self) -> String {
        self.map(|v| format!("{:.2}", v)).unwrap_or("--".to_string())
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
        let quantity = self.quantity;
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

    #[test]
    fn display_all_optionals_present() {
        let position = Position {
            quantity: 10.1,
            market_value: Some(100.5),
            symbol: AssetSymbol::new("vti"),
            unrealized_gain_today_percent: Some(-5.0),
            unrealized_gain_total_percent: Some(10.0),
            ..Default::default()
        };
        let display = position.to_string();
        let expected =
            "VTI quantity: 10.1, value: 100.50, gains today: -5.00%, gains total: 10.00%";
        assert_eq!(display, expected)
    }

    #[test]
    fn display_no_optionals_present() {
        let position = Position {
            quantity: 10.1,
            symbol: AssetSymbol::new("vti"),
            ..Default::default()
        };
        let display = position.to_string();
        let expected = "VTI quantity: 10.1, value: --, gains today: --%, gains total: --%";
        assert_eq!(display, expected)
    }
}
