use crate::platform::id::Id;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Account {
    /// The account's status.
    pub id: Id,
    /// The currently available buying power
    pub buying_power: f64,
    /// Cash balance.
    pub cash: f64,
    /// The currency the account uses.
    pub currency: String,
    /// The current number of day trades that have been made in the last
    /// five trading days (including today).
    pub daytrade_count: u64,
    /// If this account has been flagged as a day trading account or not.
    pub day_trader: bool,
    /// Real-time mark-to-market value of all long positions held in the account.
    pub market_value_long: f64,
    /// Real-time mark-to-market value of all short positions held in the account.
    pub market_value_short: f64,
    /// The sum of `cash`, `market_value_long`, and `market_value_short`.
    pub equity: f64,
}

impl Account {
    #[cfg(test)]
    pub fn fixture() -> Self {
        Self {
            buying_power: 1000.0,
            cash: 500.0,
            equity: 500.0,
            ..Default::default()
        }
    }
}

impl Display for Account {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let buying_power = self.buying_power;
        let equity = self.equity;
        write!(
            f,
            "buying power: {:.2}, equity: {:.2}",
            buying_power, equity
        )
    }
}

#[cfg(test)]
mod test {
    use crate::platform::account::Account;

    #[test]
    fn display() {
        let account = Account {
            buying_power: 100.0,
            equity: 200.0,
            ..Default::default()
        };
        let display = account.to_string();
        let expected = "buying power: 100.00, equity: 200.00";
        assert_eq!(display, expected)
    }
}
