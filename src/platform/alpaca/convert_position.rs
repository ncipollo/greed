use crate::greed_error_from;
use crate::platform::id::Id;
use crate::platform::position::Position;
use apca::RequestError;

impl From<apca::api::v2::position::Position> for Position {
    fn from(value: apca::api::v2::position::Position) -> Self {
        Self {
            id: Id::Uuid(value.asset_id.0),
            asset_class: value.asset_class.into(),
            average_entry_price: value.average_entry_price.to_f64().unwrap_or(0.0),
            change_today_percent: value.change_today,
            cost_basis: value.cost_basis,
            current_price: value.current_price.and_then(|p| p.to_f64()),
            last_day_price: value.last_day_price.and_then(|p| p.to_f64()),
            market_value: value.market_value.and_then(|p| p.to_f64()),
            quantity: value.quantity.to_f64().unwrap_or(0.0),
            quantity_available: value.quantity_available.to_f64().unwrap_or(0.0),
            side: value.side.into(),
            symbol: value.symbol.parse().unwrap(),
            unrealized_gain_today: value.unrealized_gain_today.and_then(|p| p.to_f64()),
            unrealized_gain_today_percent: value
                .unrealized_gain_today_percent
                .and_then(|p| p.to_f64())
                .map(|p| p * 100.0),
            unrealized_gain_total: value.unrealized_gain_total.and_then(|p| p.to_f64()),
            unrealized_gain_total_percent: value
                .unrealized_gain_total_percent
                .and_then(|p| p.to_f64())
                .map(|p| p * 100.0),
        }
    }
}

greed_error_from!(RequestError<apca::api::v2::positions::ListError>);
