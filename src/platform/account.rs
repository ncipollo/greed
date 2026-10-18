use crate::platform::id::Id;
use serde::Serialize;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Account {
    /// The account's status.
    pub id: Id,
    /// The currently available buying power
    pub buying_power: f64,
    /// Cash balance.
    pub cash: f64,
    /// The currency the account uses.
    pub currency: String,
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
        let cash = self.cash;
        write!(
            f,
            "buying power: {:.2}, equity: {:.2}, cash: {:.2}",
            buying_power, equity, cash
        )
    }
}

#[cfg(test)]
mod test {
    use crate::platform::account::Account;
    use serde_json::json;

    #[test]
    fn display() {
        let account = Account {
            buying_power: 100.0,
            equity: 200.0,
            cash: 50.0,
            ..Default::default()
        };
        let display = account.to_string();
        let expected = "buying power: 100.00, equity: 200.00, cash: 50.00";
        assert_eq!(display, expected)
    }

    #[test]
    fn serialize() {
        let account = Account {
            buying_power: 100.0,
            cash: 50.0,
            currency: "USD".to_string(),
            market_value_long: 10.0,
            market_value_short: 5.0,
            equity: 200.0,
            ..Default::default()
        };

        let json = serde_json::to_value(&account).expect("failed to serialize");
        let expected = json!({
            "id": "00000000-0000-0000-0000-000000000000",
            "buying_power": 100.0,
            "cash": 50.0,
            "currency": "USD",
            "market_value_long": 10.0,
            "market_value_short": 5.0,
            "equity": 200.0,
        });

        assert_eq!(json, expected)
    }
}
