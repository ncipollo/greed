use crate::asset::AssetSymbol;
use crate::platform::asset_class::AssetClass;
use crate::platform::id::Id;
use crate::platform::order::amount::Amount;
use crate::platform::order::class::OrderClass;
use crate::platform::order::order_type::OrderType;
use crate::platform::order::time_in_force::TimeInForce;
use crate::platform::side::Side;
use chrono::{DateTime, Utc};
use num_decimal::Num;
use crate::platform::order::status::Status;

pub mod amount;
pub mod class;
pub mod order_type;
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
    pub symbol: AssetSymbol,
    pub amount: Amount,
    pub filled_quantity: Num,
    pub order_type: OrderType,
    /// The order class.
    pub class: OrderClass,
    /// The side the order is on.
    pub side: Side,
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
