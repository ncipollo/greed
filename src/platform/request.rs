pub mod stop_loss;

use crate::platform::order::class::OrderClass;
use crate::platform::order::order_type::OrderType;
use crate::platform::order::time_in_force::TimeInForce;
use num_decimal::Num;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct OrderRequest {
    pub class: OrderClass,
    pub order_type: OrderType,
    pub time_in_force: TimeInForce,
    pub limit_price: Option<Num>,
    pub stop_price: Option<Num>,
    pub trail_price: Option<Num>,
    pub trail_percent: Option<Num>,
    pub extended_hours: bool,
}
