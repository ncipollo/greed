use apca::api::v2::order::{OrderReq, OrderReqInit};
use clap::builder::TypedValueParser;
use crate::platform::request::OrderRequest;

mod convert_stop_loss;
mod convert_take_profit;

// impl From<OrderRequest> for OrderReq {
//     fn from(value: OrderRequest) -> Self {
//         OrderReq {
//             symbol: value.symbol.to_string().parse().parse(),
//             amount: value.amount.into(),
//             side: Side::Buy,
//             class: Default::default(),
//             type_: Default::default(),
//             time_in_force: Default::default(),
//             limit_price: None,
//             stop_price: None,
//             trail_price: None,
//             trail_percent: None,
//             take_profit: None,
//             stop_loss: None,
//             extended_hours: false,
//             client_order_id: None,
//         }
//     }
// }