use crate::platform::id::Id;
use crate::platform::quote::Quote;
use num_decimal::Num;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Account {
    /// The account's status.
    pub id: Id,
    /// The currently available buying power
    pub buying_power: Num,
    /// Cash balance.
    pub cash: Num,
    /// The currency the account uses.
    pub currency: String,
    /// The current number of day trades that have been made in the last
    /// five trading days (including today).
    pub daytrade_count: u64,
    /// If this account has been flagged as a day trading account or not.
    pub day_trader: bool,
    /// Real-time mark-to-market value of all long positions held in the account.
    pub market_value_long: Num,
    /// Real-time mark-to-market value of all short positions held in the account.
    pub market_value_short: Num,
    /// The sum of `cash`, `market_value_long`, and `market_value_short`.
    pub equity: Num,
}

impl Display for Account {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let buying_power = self.buying_power.to_f64().unwrap_or(0.0);
        let equity = self.equity.to_f64().unwrap_or(0.0);
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
    use num_decimal::Num;

    #[test]
    fn display() {
        let account = Account {
            buying_power: Num::from(100),
            equity: Num::from(200),
            ..Default::default()
        };
        let display = account.to_string();
        let expected = "buying power: 100.00, equity: 200.00";
        assert_eq!(display, expected)
    }
}
