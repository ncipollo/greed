use crate::greed_error_from;
use crate::platform::request::OrderRequest;
use apca::api::v2::order::CreateReq;
use apca::RequestError;

mod convert_stop_loss;
mod convert_take_profit;

impl From<OrderRequest> for CreateReq {
    fn from(value: OrderRequest) -> Self {
        CreateReq {
            symbol: value.symbol.into(),
            amount: value.amount.into(),
            side: value.side.into(),
            class: value.class.into(),
            type_: value.order_type.into(),
            time_in_force: value.time_in_force.into(),
            limit_price: value.limit_price,
            stop_price: value.stop_price,
            trail_price: value.trail_price,
            trail_percent: value.trail_percent,
            take_profit: value.take_profit.map(|val| val.into()),
            stop_loss: value.stop_loss.map(|val| val.into()),
            extended_hours: value.extended_hours,
            client_order_id: None,
            _non_exhaustive: (),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::asset::AssetSymbol;
    use crate::platform::order::amount::Amount;
    use crate::platform::order::class::OrderClass;
    use crate::platform::order::order_type::OrderType;
    use crate::platform::order::side::OrderSide;
    use crate::platform::order::time_in_force::TimeInForce;
    use crate::platform::request::stop_loss::StopLoss;
    use crate::platform::request::take_profit::TakeProfit;
    use crate::platform::request::OrderRequest;
    use apca::api::v2::asset::Symbol;
    use apca::api::v2::order::{Class, CreateReq, Side, Type};
    use num_decimal::Num;

    #[test]
    fn into_alpaca() {
        let request = OrderRequest {
            symbol: AssetSymbol::new("VTI"),
            class: OrderClass::Simple,
            order_type: OrderType::Limit,
            amount: Amount::Notional(1.0),
            side: OrderSide::Buy,
            time_in_force: TimeInForce::Day,
            limit_price: Some(Num::from(2)),
            stop_price: Some(Num::from(3)),
            trail_price: Some(Num::from(4)),
            trail_percent: Some(Num::from(5)),
            take_profit: Some(TakeProfit::Limit(Num::from(6))),
            stop_loss: Some(StopLoss::Stop(Num::from(7))),
            extended_hours: true,
        };
        let expected = CreateReq {
            symbol: Symbol::Sym("VTI".to_string()),
            amount: apca::api::v2::order::Amount::Notional {
                notional: Num::from(1),
            },
            side: Side::Buy,
            class: Class::Simple,
            type_: Type::Limit,
            time_in_force: apca::api::v2::order::TimeInForce::Day,
            limit_price: Some(Num::from(2)),
            stop_price: Some(Num::from(3)),
            trail_price: Some(Num::from(4)),
            trail_percent: Some(Num::from(5)),
            take_profit: Some(apca::api::v2::order::TakeProfit::Limit(Num::from(6))),
            stop_loss: Some(apca::api::v2::order::StopLoss::Stop(Num::from(7))),
            extended_hours: true,
            client_order_id: None,
            _non_exhaustive: (),
        };
        let alpaca_request: CreateReq = request.into();
        assert_eq!(alpaca_request, expected)
    }
}

greed_error_from!(RequestError<apca::api::v2::order::CreateError>);
