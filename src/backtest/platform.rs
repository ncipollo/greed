use crate::asset::AssetSymbol;
use crate::backtest::market_data::MarketData;
use crate::backtest::portfolio::Portfolio;
use crate::clock::{Clock, SimulatedClock};
use crate::error::GreedError;
use crate::num::NumFromFloat;
use crate::platform::account::Account;
use crate::platform::bar::bar_request::BarRequest;
use crate::platform::bars::Bars;
use crate::platform::id::Id;
use crate::platform::order::amount::Amount;
use crate::platform::order::order_type::OrderType;
use crate::platform::order::side::OrderSide;
use crate::platform::order::status::Status;
use crate::platform::order::Order;
use crate::platform::position::Position;
use crate::platform::quote::Quote;
use crate::platform::request::OrderRequest;
use crate::platform::side::Side;
use crate::platform::FinancialPlatform;
use async_trait::async_trait;
use num_decimal::Num;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// A simulated brokerage: prices come from stored historical bars at the
/// simulated clock's current time, and the account lives in memory.
///
/// v1 order support: market orders fill immediately at the latest close.
/// Limit, stop, stop-limit and trailing-stop orders, and bracket legs
/// (`stop_loss`/`take_profit`), are rejected as unsupported rather than
/// silently mis-simulated.
pub struct BacktestPlatform {
    clock: Arc<SimulatedClock>,
    market: MarketData,
    portfolio: Mutex<Portfolio>,
}

impl BacktestPlatform {
    pub fn new(clock: Arc<SimulatedClock>, market: MarketData, starting_cash: f64) -> Self {
        Self {
            clock,
            market,
            portfolio: Mutex::new(Portfolio::new(starting_cash)),
        }
    }

    fn price_of(&self, symbol: &AssetSymbol) -> Option<f64> {
        self.market
            .latest_bar(symbol, self.clock.now())
            .map(|bar| bar.close)
    }

    fn reject_unsupported(request: &OrderRequest) -> Result<(), GreedError> {
        if request.order_type != OrderType::Market {
            return Err(GreedError::new(&format!(
                "{} orders are unsupported in backtest (only market orders)",
                request.order_type
            )));
        }
        if request.stop_loss.is_some() || request.take_profit.is_some() {
            return Err(GreedError::new(
                "stop loss and take profit legs are unsupported in backtest",
            ));
        }
        Ok(())
    }
}

#[async_trait]
impl FinancialPlatform for BacktestPlatform {
    async fn account(&self) -> Result<Account, GreedError> {
        let portfolio = self.portfolio.lock().unwrap();
        let cash = portfolio.cash();
        let equity = portfolio.equity(|symbol| self.price_of(symbol));
        let market_value_long = equity - cash;
        Ok(Account {
            id: Id::default(),
            buying_power: cash,
            cash,
            currency: "USD".to_string(),
            market_value_long,
            market_value_short: 0.0,
            equity,
        })
    }

    async fn bars(&self, request: BarRequest) -> Result<Bars, GreedError> {
        let mut bars = self.market.bars(
            &request.symbol,
            &request.timeframe,
            request.start,
            request.end,
            self.clock.now(),
        );
        if let Some(limit) = request.limit {
            let excess = bars.len().saturating_sub(limit);
            bars.drain(..excess);
        }
        Ok(Bars {
            symbol: request.symbol,
            bars,
        })
    }

    async fn latest_quotes(&self, symbols: &[AssetSymbol]) -> Result<Vec<Quote>, GreedError> {
        let now = self.clock.now();
        Ok(symbols
            .iter()
            .filter_map(|symbol| {
                let bar = self.market.latest_bar(symbol, now)?;
                Some(Quote {
                    time: bar.timestamp,
                    ask_price: bar.close,
                    ask_size: 0,
                    bid_price: bar.close,
                    bid_size: 0,
                    symbol: symbol.clone(),
                })
            })
            .collect())
    }

    async fn place_order(&self, request: OrderRequest) -> Result<Order, GreedError> {
        Self::reject_unsupported(&request)?;
        let price = self.price_of(&request.symbol).ok_or_else(|| {
            GreedError::new(&format!("no current price for {}", request.symbol.symbol))
        })?;
        let quantity = match request.amount {
            Amount::Quantity(quantity) => quantity,
            Amount::Notional(notional) => notional / price,
        };
        let mut portfolio = self.portfolio.lock().unwrap();
        match request.side {
            OrderSide::Buy => portfolio.buy(&request.symbol, quantity, price)?,
            OrderSide::Sell => portfolio.sell(&request.symbol, quantity, price)?,
        }
        let now = self.clock.now();
        let order = Order {
            id: Id::Uuid(Uuid::new_v4()),
            status: Status::Filled,
            created_at: now,
            submitted_at: Some(now),
            filled_at: Some(now),
            symbol: request.symbol,
            amount: request.amount,
            filled_quantity: Num::from_f64(quantity),
            order_type: OrderType::Market,
            side: request.side,
            time_in_force: request.time_in_force,
            average_fill_price: Some(Num::from_f64(price)),
            ..Default::default()
        };
        portfolio.record_order(order.clone());
        Ok(order)
    }

    async fn positions(&self) -> Result<Vec<Position>, GreedError> {
        let portfolio = self.portfolio.lock().unwrap();
        let mut positions: Vec<Position> = portfolio
            .holdings()
            .iter()
            .map(|(symbol, holding)| {
                let price = self.price_of(symbol);
                Position {
                    average_entry_price: holding.average_cost,
                    cost_basis: Num::from_f64(holding.average_cost * holding.quantity),
                    current_price: price,
                    market_value: price.map(|p| p * holding.quantity),
                    quantity: holding.quantity,
                    quantity_available: holding.quantity,
                    side: Side::Long,
                    symbol: symbol.clone(),
                    unrealized_gain_total: price
                        .map(|p| (p - holding.average_cost) * holding.quantity),
                    ..Default::default()
                }
            })
            .collect();
        positions.sort_by(|a, b| a.symbol.symbol.cmp(&b.symbol.symbol));
        Ok(positions)
    }

    /// Market orders fill immediately, so nothing is ever left open.
    async fn open_orders(&self) -> Result<Vec<Order>, GreedError> {
        Ok(Vec::new())
    }

    async fn recent_orders(&self) -> Result<Vec<Order>, GreedError> {
        Ok(self.portfolio.lock().unwrap().orders().to_vec())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::platform::bar::time_frame::TimeFrame;
    use crate::platform::bar::Bar;
    use chrono::{DateTime, Duration, TimeZone, Utc};
    use std::collections::HashMap;

    fn at(day: u32, hour: u32, minute: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, day, hour, minute, 0).unwrap()
    }

    fn vti() -> AssetSymbol {
        AssetSymbol::new("VTI")
    }

    fn bar(time: DateTime<Utc>, close: f64) -> Bar {
        Bar {
            timestamp: time,
            open: close,
            high: close,
            low: close,
            close,
            volume: 1,
        }
    }

    /// VTI closes at $100 at 14:30, $110 at 14:31 and $120 at 14:32 on Monday.
    fn platform_at(now: DateTime<Utc>, cash: f64) -> (BacktestPlatform, Arc<SimulatedClock>) {
        let market = MarketData::new(HashMap::from([(
            vti(),
            vec![
                bar(at(5, 14, 30), 100.0),
                bar(at(5, 14, 31), 110.0),
                bar(at(5, 14, 32), 120.0),
            ],
        )]));
        let clock = Arc::new(SimulatedClock::new(now));
        (BacktestPlatform::new(clock.clone(), market, cash), clock)
    }

    #[tokio::test]
    async fn quotes_use_the_latest_close_as_bid_and_ask() {
        let (platform, _) = platform_at(at(5, 14, 31), 1000.0);

        let quotes = platform.latest_quotes(&[vti()]).await.unwrap();

        assert_eq!(quotes.len(), 1);
        assert_eq!(quotes[0].ask_price, 110.0);
        assert_eq!(quotes[0].bid_price, 110.0);
    }

    #[tokio::test]
    async fn quotes_move_with_the_clock() {
        let (platform, clock) = platform_at(at(5, 14, 30), 1000.0);

        clock.advance(Duration::minutes(2));
        let quotes = platform.latest_quotes(&[vti()]).await.unwrap();

        assert_eq!(quotes[0].ask_price, 120.0);
    }

    #[tokio::test]
    async fn stale_symbols_get_no_quote() {
        let (platform, _) = platform_at(at(7, 14, 30), 1000.0);

        let quotes = platform.latest_quotes(&[vti()]).await.unwrap();

        assert!(quotes.is_empty());
    }

    #[tokio::test]
    async fn bars_never_come_from_after_the_clock() {
        let (platform, _) = platform_at(at(5, 14, 31), 1000.0);

        let bars = platform
            .bars(BarRequest {
                symbol: vti(),
                limit: None,
                start: at(5, 0, 0),
                end: at(6, 0, 0),
                timeframe: TimeFrame::OneMinute,
            })
            .await
            .unwrap();

        assert_eq!(bars.bars.len(), 2);
        assert!(bars.bars.iter().all(|b| b.timestamp <= at(5, 14, 31)));
    }

    #[tokio::test]
    async fn bars_limit_keeps_the_most_recent() {
        let (platform, _) = platform_at(at(5, 14, 32), 1000.0);

        let bars = platform
            .bars(BarRequest {
                symbol: vti(),
                limit: Some(1),
                start: at(5, 0, 0),
                end: at(6, 0, 0),
                timeframe: TimeFrame::OneMinute,
            })
            .await
            .unwrap();

        assert_eq!(bars.bars.len(), 1);
        assert_eq!(bars.bars[0].close, 120.0);
    }

    #[tokio::test]
    async fn market_buy_by_quantity_fills_at_the_current_close() {
        let (platform, _) = platform_at(at(5, 14, 30), 1000.0);

        let order = platform
            .place_order(OrderRequest::market_order_buy(vti(), Amount::Quantity(2.0)))
            .await
            .unwrap();

        assert_eq!(order.status, Status::Filled);
        assert_eq!(order.filled_quantity, Num::from_f64(2.0));
        assert_eq!(order.average_fill_price, Some(Num::from_f64(100.0)));
        let account = platform.account().await.unwrap();
        assert_eq!(account.cash, 800.0);
        assert_eq!(account.market_value_long, 200.0);
        assert_eq!(account.equity, 1000.0);
    }

    #[tokio::test]
    async fn notional_buy_converts_to_fractional_quantity() {
        let (platform, _) = platform_at(at(5, 14, 30), 1000.0);

        platform
            .place_order(OrderRequest::market_order_buy(
                vti(),
                Amount::Notional(50.0),
            ))
            .await
            .unwrap();

        let positions = platform.positions().await.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].quantity, 0.5);
        assert_eq!(platform.account().await.unwrap().cash, 950.0);
    }

    #[tokio::test]
    async fn equity_marks_positions_at_the_current_price() {
        let (platform, clock) = platform_at(at(5, 14, 30), 1000.0);
        platform
            .place_order(OrderRequest::market_order_buy(vti(), Amount::Quantity(2.0)))
            .await
            .unwrap();

        clock.advance(Duration::minutes(2));

        let account = platform.account().await.unwrap();
        assert_eq!(account.equity, 800.0 + 240.0);
        let position = &platform.positions().await.unwrap()[0];
        assert_eq!(position.unrealized_gain_total, Some(40.0));
    }

    #[tokio::test]
    async fn buying_more_than_buying_power_is_rejected() {
        let (platform, _) = platform_at(at(5, 14, 30), 100.0);

        let result = platform
            .place_order(OrderRequest::market_order_buy(vti(), Amount::Quantity(2.0)))
            .await;

        assert!(result.is_err());
        assert_eq!(platform.account().await.unwrap().cash, 100.0);
        assert!(platform.recent_orders().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn selling_more_than_held_is_rejected() {
        let (platform, _) = platform_at(at(5, 14, 30), 1000.0);

        let result = platform
            .place_order(OrderRequest::market_order_sell(
                vti(),
                Amount::Quantity(1.0),
            ))
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn selling_returns_cash_at_the_current_price() {
        let (platform, clock) = platform_at(at(5, 14, 30), 1000.0);
        platform
            .place_order(OrderRequest::market_order_buy(vti(), Amount::Quantity(2.0)))
            .await
            .unwrap();
        clock.advance(Duration::minutes(2));

        platform
            .place_order(OrderRequest::market_order_sell(
                vti(),
                Amount::Quantity(2.0),
            ))
            .await
            .unwrap();

        assert_eq!(platform.account().await.unwrap().cash, 1040.0);
        assert!(platform.positions().await.unwrap().is_empty());
        assert_eq!(platform.recent_orders().await.unwrap().len(), 2);
        assert!(platform.open_orders().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn limit_orders_are_rejected_as_unsupported() {
        let (platform, _) = platform_at(at(5, 14, 30), 1000.0);
        let request = OrderRequest {
            symbol: vti(),
            order_type: OrderType::Limit,
            amount: Amount::Quantity(1.0),
            limit_price: Some(Num::from_f64(99.0)),
            ..Default::default()
        };

        let error = platform.place_order(request).await.unwrap_err().to_string();

        assert!(error.contains("unsupported in backtest"), "{error}");
    }

    #[tokio::test]
    async fn orders_without_a_price_are_rejected() {
        let (platform, _) = platform_at(at(5, 14, 0), 1000.0);

        let result = platform
            .place_order(OrderRequest::market_order_buy(vti(), Amount::Quantity(1.0)))
            .await;

        assert!(result.is_err());
    }

    #[test]
    fn rate_limit_is_none() {
        let (platform, _) = platform_at(at(5, 14, 30), 1000.0);

        assert!(platform.rate_limit().is_none());
    }
}
