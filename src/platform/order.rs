use crate::asset::AssetSymbol;
use crate::platform::asset_class::AssetClass;
use crate::platform::id::Id;
use crate::platform::order::amount::Amount;
use crate::platform::order::class::OrderClass;
use crate::platform::order::order_type::OrderType;
use crate::platform::order::status::Status;
use crate::platform::order::time_in_force::TimeInForce;
use chrono::{DateTime, Utc};
use num_decimal::Num;
use std::fmt::{Display, Formatter};
use crate::platform::order::side::OrderSide;

pub mod amount;
pub mod class;
pub mod order_type;
pub mod side;
pub mod status;
pub mod time_in_force;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
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
    #[cfg(test)]
    pub fn fixture(symbol: AssetSymbol) -> Self {
        Self {
            amount: Amount::Quantity(Num::from(1)),
            symbol,
            ..Default::default()
        }
    }
}

impl Display for Order {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} of {}",
            self.side, self.amount, self.symbol
        )
    }
}

#[cfg(test)]
mod test {
    use num_decimal::Num;
    use crate::asset::AssetSymbol;
    use crate::platform::order::amount::Amount;
    use crate::platform::order::Order;
    use crate::platform::order::side::OrderSide;

    #[test]
    fn display() {
        let order = Order {
            amount: Amount::Quantity(Num::from(10)),
            side: OrderSide::Buy,
            symbol: AssetSymbol::new("VTI"),
            ..Default::default()
        };
        let display = order.to_string();
        assert_eq!(display, "buy 10.00 units of VTI")
    }
}