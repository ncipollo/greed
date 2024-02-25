use crate::asset::AssetSymbol;
use crate::platform::order::amount::Amount;
use crate::platform::request::OrderRequest;
use num_decimal::Num;

#[derive(Debug, PartialEq)]
pub enum Action {
    Buy { symbol: AssetSymbol, amount: Amount },
    Sell { symbol: AssetSymbol, amount: Amount },
}

impl Action {
    pub fn buy_percent(symbol: AssetSymbol, percent: Num) -> Self {
        let amount = Amount::Notional(percent);
        Self::Buy { symbol, amount }
    }

    pub fn sell_percent(symbol: AssetSymbol, percent: Num) -> Self {
        let amount = Amount::Notional(percent);
        Self::Sell { symbol, amount }
    }

    pub fn into_request(self) -> OrderRequest {
        match self {
            Action::Buy { symbol, amount } => OrderRequest::market_order_buy(symbol, amount),
            Action::Sell { symbol, amount } => OrderRequest::market_order_sell(symbol, amount),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_request_buy() {
        let action = Action::buy_percent(AssetSymbol::new("VTI"), Num::from(10));
        let request = action.into_request();
        let expected = OrderRequest::market_order_buy(
            AssetSymbol::new("VTI"),
            Amount::Notional(Num::from(10)),
        );
        assert_eq!(expected, request);
    }

    #[test]
    fn into_request_sell() {
        let action = Action::sell_percent(AssetSymbol::new("VTI"), Num::from(10));
        let request = action.into_request();
        let expected = OrderRequest::market_order_sell(
            AssetSymbol::new("VTI"),
            Amount::Notional(Num::from(10)),
        );
        assert_eq!(expected, request);
    }
}
