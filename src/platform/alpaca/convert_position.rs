use crate::greed_error_from;
use crate::platform::id::Id;
use crate::platform::position::Position;
use apca::RequestError;

impl From<apca::api::v2::position::Position> for Position {
    fn from(value: apca::api::v2::position::Position) -> Self {
        Self {
            id: Id::Uuid(value.asset_id.0),
            asset_class: value.asset_class.into(),
            average_entry_price: value.average_entry_price,
            change_today_percent: value.change_today,
            cost_basis: value.cost_basis,
            current_price: value.current_price,
            last_day_price: value.last_day_price,
            market_value: value.market_value,
            quantity: value.quantity,
            quantity_available: value.quantity_available,
            side: value.side.into(),
            symbol: value.symbol.parse().unwrap(),
            unrealized_gain_today: value.unrealized_gain_today,
            unrealized_gain_today_percent: value.unrealized_gain_today_percent,
            unrealized_gain_total: value.unrealized_gain_total,
            unrealized_gain_total_percent: value.unrealized_gain_total_percent,
        }
    }
}

greed_error_from!(RequestError<apca::api::v2::positions::ListError>);
