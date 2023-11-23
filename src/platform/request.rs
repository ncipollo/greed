pub mod stop_loss;
pub mod take_profit;

use crate::platform::order::class::OrderClass;
use crate::platform::order::order_type::OrderType;
use crate::platform::order::time_in_force::TimeInForce;
use num_decimal::Num;
use crate::platform::order::amount::Amount;
use crate::platform::order::side::OrderSide;
use crate::platform::request::stop_loss::StopLoss;
use crate::platform::request::take_profit::TakeProfit;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct OrderRequest {
    pub class: OrderClass,
    pub order_type: OrderType,
    pub amount: Amount,
    pub side: OrderSide,
    pub time_in_force: TimeInForce,
    pub limit_price: Option<Num>,
    pub stop_price: Option<Num>,
    pub trail_price: Option<Num>,
    pub trail_percent: Option<Num>,
    pub take_profit: Option<TakeProfit>,
    pub stop_loss: Option<StopLoss>,
    pub extended_hours: bool,
}
