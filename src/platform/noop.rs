use async_trait::async_trait;
use crate::asset::AssetSymbol;
use crate::error::GreedError;
use crate::platform::account::Account;
use crate::platform::bar::bar_request::BarRequest;
use crate::platform::bars::Bars;
use crate::platform::FinancialPlatform;
use crate::platform::order::Order;
use crate::platform::position::Position;
use crate::platform::quote::Quote;
use crate::platform::request::OrderRequest;

struct NoOpPlatform;

impl NoOpPlatform {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl FinancialPlatform for NoOpPlatform {
    async fn account(&self) -> Result<Account, GreedError> {
        Ok(Account::default())
    }

    async fn bars(&self, _bars_request: BarRequest) -> Result<Bars, GreedError> {
        Ok(Bars::default())
    }

    async fn latest_quotes(&self, _symbols: &Vec<AssetSymbol>) -> Result<Vec<Quote>, GreedError> {
        Ok(Vec::new())
    }

    async fn place_order(&self, _order_request: OrderRequest) -> Result<Order, GreedError> {
        Ok(Order::default())
    }

    async fn positions(&self) -> Result<Vec<Position>, GreedError> {
        Ok(Vec::new())
    }

    async fn open_orders(&self) -> Result<Vec<Order>, GreedError> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_account() {
        let platform = NoOpPlatform::new();
        let result = platform.account().await;
        assert_eq!(result.unwrap(), Account::default());
    }

    #[tokio::test]
    async fn test_bars() {
        let platform = NoOpPlatform::new();
        let bars_request = BarRequest::default();
        let result = platform.bars(bars_request).await;
        assert_eq!(result.unwrap(), Bars::default());
    }

    #[tokio::test]
    async fn test_latest_quotes() {
        let platform = NoOpPlatform::new();
        let symbols = vec![];
        let result = platform.latest_quotes(&symbols).await;
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_place_order() {
        let platform = NoOpPlatform::new();
        let order_request = OrderRequest::default();
        let result = platform.place_order(order_request).await;
        assert_eq!(result.unwrap(), Order::default());
    }

    #[tokio::test]
    async fn test_positions() {
        let platform = NoOpPlatform::new();
        let result = platform.positions().await;
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_open_orders() {
        let platform = NoOpPlatform::new();
        let result = platform.open_orders().await;
        assert!(result.unwrap().is_empty());
    }
}