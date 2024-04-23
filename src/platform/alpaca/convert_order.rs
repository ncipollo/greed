use crate::greed_error_from;
use crate::platform::id::Id;
use crate::platform::order::Order;
use apca::RequestError;

mod convert_amount;
mod convert_class;
mod convert_status;
mod convert_time;
mod convert_type;

impl From<apca::api::v2::order::Order> for Order {
    fn from(value: apca::api::v2::order::Order) -> Self {
        Self {
            id: Id::Uuid(value.id.0),
            status: value.status.into(),
            created_at: value.created_at,
            updated_at: value.updated_at,
            submitted_at: value.submitted_at,
            filled_at: value.filled_at,
            expired_at: value.expired_at,
            canceled_at: value.canceled_at,
            asset_class: value.asset_class.into(),
            symbol: value.symbol.parse().unwrap(),
            amount: value.amount.into(),
            filled_quantity: value.filled_quantity,
            order_type: value.type_.into(),
            class: value.class.into(),
            side: value.side.into(),
            time_in_force: value.time_in_force.into(),
            limit_price: value.limit_price,
            stop_price: value.stop_price,
            trail_price: value.trail_price,
            trail_percent: value.trail_percent,
            average_fill_price: value.average_fill_price,
            extended_hours: value.extended_hours,
        }
    }
}

greed_error_from!(RequestError<apca::api::v2::orders::ListError>);
