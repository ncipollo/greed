use crate::asset::AssetSymbol;
use crate::platform::asset_class::AssetClass;
use crate::platform::id::Id;
use crate::platform::order::amount::Amount;
use crate::platform::order::class::OrderClass;
use crate::platform::order::order_type::OrderType;
use crate::platform::order::side::OrderSide;
use crate::platform::order::status::Status;
use crate::platform::order::time_in_force::TimeInForce;
use chrono::{DateTime, Utc};
use num_decimal::Num;
use std::fmt::{Display, Formatter};

pub mod amount;
pub mod class;
pub mod order_type;
pub mod side;
pub mod status;
pub mod time_in_force;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Order {
    /// The identifier associated with the order.
    pub id: Id,
    /// The status of the order.
    pub status: Status,
    /// Timestamp for when this order was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp this order was updated at last.
    pub updated_at: Option<DateTime<Utc>>,
    /// Timestamp this order was submitted at.
    pub submitted_at: Option<DateTime<Utc>>,
    /// Timestamp this order was filled at.
    pub filled_at: Option<DateTime<Utc>>,
    /// Timestamp this order expired at.
    pub expired_at: Option<DateTime<Utc>>,
    /// Timestamp this order was cancelled.
    pub canceled_at: Option<DateTime<Utc>>,
    /// The order's asset class.
    pub asset_class: AssetClass,
    /// The symbol of the asset being traded in this order.
    pub symbol: AssetSymbol,
    /// The amount of assets we are trading in this order.
    pub amount: Amount,
    /// The amount we've filled in this order.
    pub filled_quantity: Num,
    /// The type of order.
    pub order_type: OrderType,
    /// The order class.
    pub class: OrderClass,
    /// The side the order is on.
    pub side: OrderSide,
    /// A representation of how long the order will be valid.
    pub time_in_force: TimeInForce,
    /// The limit price.
    pub limit_price: Option<Num>,
    /// The stop price.
    pub stop_price: Option<Num>,
    /// The dollar value away from the high water mark.
    pub trail_price: Option<Num>,
    /// The percent value away from the high water mark.
    pub trail_percent: Option<Num>,
    /// The average price at which the order was filled.
    pub average_fill_price: Option<Num>,
    /// If true, the order is eligible for execution outside regular
    /// trading hours.
    pub extended_hours: bool,
}

impl Order {
    /// Calculate the estimated value of this order given the ask price. Primarily useful for an
    /// open order.
    pub fn estimated_value(&self, ask_price: f64) -> f64 {
        match &self.amount {
            Amount::Quantity(value) => value * ask_price,
            Amount::Notional(value) => *value,
        }
    }
    #[cfg(test)]
    pub fn fixture(symbol: AssetSymbol) -> Self {
        Self {
            amount: Amount::Quantity(1.0),
            symbol,
            ..Default::default()
        }
    }
}

impl Display for Order {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let submitted_at = self
            .submitted_at
            .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
            .map(|s| s + " ")
            .unwrap_or("".to_string());
        write!(
            f,
            "{}{} {} of {}",
            submitted_at, self.side, self.amount, self.symbol
        )
    }
}

#[cfg(test)]
mod test {
    use crate::asset::AssetSymbol;
    use crate::date::DateTimeFixture;
    use crate::platform::order::amount::Amount;
    use crate::platform::order::side::OrderSide;
    use crate::platform::order::Order;

    #[test]
    fn estimated_value_quantity() {
        let order = Order {
            amount: Amount::Quantity(10.0),
            ..Default::default()
        };
        let estimated_value = order.estimated_value(100.0);
        assert_eq!(estimated_value, 1000.0)
    }

    #[test]
    fn estimated_value_notational() {
        let order = Order {
            amount: Amount::Notional(10.0),
            ..Default::default()
        };
        let estimated_value = order.estimated_value(100.0);
        assert_eq!(estimated_value, 10.0)
    }

    #[test]
    fn display_no_submission_time() {
        let order = Order {
            amount: Amount::Quantity(10.0),
            side: OrderSide::Buy,
            symbol: AssetSymbol::new("VTI"),
            ..Default::default()
        };
        let display = order.to_string();
        assert_eq!(display, "buy 10.00 units of VTI")
    }

    #[test]
    fn display_with_submission_time() {
        let order = Order {
            amount: Amount::Quantity(10.0),
            side: OrderSide::Buy,
            symbol: AssetSymbol::new("VTI"),
            submitted_at: Some(DateTimeFixture::utc()),
            ..Default::default()
        };
        let display = order.to_string();
        assert_eq!(display, "2023-12-04 08:00 buy 10.00 units of VTI")
    }
}
