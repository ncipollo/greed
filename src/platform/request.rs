pub mod stop_loss;
pub mod take_profit;

use crate::platform::order::amount::Amount;
use crate::platform::order::class::OrderClass;
use crate::platform::order::order_type::OrderType;
use crate::platform::order::side::OrderSide;
use crate::platform::order::time_in_force::TimeInForce;
use crate::platform::request::stop_loss::StopLoss;
use crate::platform::request::take_profit::TakeProfit;
use num_decimal::Num;
use std::fmt::{Display, Formatter};
use crate::asset::AssetSymbol;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OrderRequest {
    pub symbol: AssetSymbol,
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

impl OrderRequest {
    pub fn market_order_buy(symbol: AssetSymbol, amount: Amount) -> Self {
        Self {
            symbol,
            amount,
            side: OrderSide::Buy,
            ..Default::default()
        }
    }

    pub fn market_order_sell(symbol: AssetSymbol, amount: Amount) -> Self {
        Self {
            symbol,
            amount,
            side: OrderSide::Sell,
            ..Default::default()
        }
    }
}

impl Display for OrderRequest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {} of {}",
            self.order_type, self.side, self.amount, self.symbol
        )
    }
}

#[cfg(test)]
mod test {
    use num_decimal::Num;
    use crate::asset::AssetSymbol;
    use crate::platform::order::amount::Amount;
    use crate::platform::order::side::OrderSide;
    use crate::platform::request::OrderRequest;

    #[test]
    fn display() {
        let request = OrderRequest::market_order_buy("VTI".parse().unwrap(),
                                                 Amount::Quantity(1.into()));
        let display = request.to_string();
        assert_eq!(display, "market buy 1.00 units of VTI")
    }

    #[test]
    fn market_order_buy() {
        let request = OrderRequest::market_order_buy("VTI".parse().unwrap(),
                                                     Amount::Quantity(1.into()));
        let expected = OrderRequest {
            symbol: AssetSymbol::new("VTI"),
            amount: Amount::Quantity(Num::from(1)),
            side: OrderSide::Buy,
            ..Default::default()
        };
        assert_eq!(expected, request)
    }

    #[test]
    fn market_order_sell() {
        let request = OrderRequest::market_order_sell("VTI".parse().unwrap(),
                                                     Amount::Quantity(1.into()));
        let expected = OrderRequest {
            symbol: AssetSymbol::new("VTI"),
            amount: Amount::Quantity(Num::from(1)),
            side: OrderSide::Sell,
            ..Default::default()
        };
        assert_eq!(expected, request)
    }
}
