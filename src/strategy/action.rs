use std::fmt::{Display, Formatter};
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
    pub fn buy_notional(symbol: AssetSymbol, notional: Num) -> Self {
        let amount = Amount::Notional(notional);
        Self::Buy { symbol, amount }
    }

    #[allow(dead_code)]
    pub fn sell_notional(symbol: AssetSymbol, notional: Num) -> Self {
        let amount = Amount::Notional(notional);
        Self::Sell { symbol, amount }
    }

    pub fn sell_quantity(symbol: AssetSymbol, quantity: Num) -> Self {
        let amount = Amount::Quantity(quantity);
        Self::Sell { symbol, amount }
    }

    pub fn into_request(self) -> OrderRequest {
        match self {
            Action::Buy { symbol, amount } => OrderRequest::market_order_buy(symbol, amount),
            Action::Sell { symbol, amount } => OrderRequest::market_order_sell(symbol, amount),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Action::Buy {
                symbol: _symbol,
                amount,
            } => amount.is_empty(),
            Action::Sell {
                symbol: _symbol,
                amount,
            } => amount.is_empty(),
        }
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Buy { symbol, amount } => {
                write!(f, "💸buy {} - amount: {}", symbol, amount)
            }
            Action::Sell { symbol, amount } => {
                write!(f, "💰sell {} - amount: {}", symbol, amount)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_request_buy() {
        let action = Action::buy_notional(AssetSymbol::new("VTI"), Num::from(10));
        let request = action.into_request();
        let expected = OrderRequest::market_order_buy(
            AssetSymbol::new("VTI"),
            Amount::Notional(Num::from(10)),
        );
        assert_eq!(expected, request);
    }

    #[test]
    fn into_request_sell() {
        let action = Action::sell_notional(AssetSymbol::new("VTI"), Num::from(10));
        let request = action.into_request();
        let expected = OrderRequest::market_order_sell(
            AssetSymbol::new("VTI"),
            Amount::Notional(Num::from(10)),
        );
        assert_eq!(expected, request);
    }

    #[test]
    fn is_empty_buy_empty() {
        let action = Action::buy_notional(AssetSymbol::new("VTI"), Num::from(0));
        assert!(action.is_empty())
    }

    #[test]
    fn is_empty_buy_not_empty() {
        let action = Action::buy_notional(AssetSymbol::new("VTI"), Num::from(1));
        assert!(!action.is_empty())
    }

    #[test]
    fn is_empty_sell_empty() {
        let action = Action::sell_notional(AssetSymbol::new("VTI"), Num::from(0));
        assert!(action.is_empty())
    }

    #[test]
    fn is_empty_sell_not_empty() {
        let action = Action::sell_notional(AssetSymbol::new("VTI"), Num::from(1));
        assert!(!action.is_empty())
    }
}
