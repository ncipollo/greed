use crate::asset::AssetSymbol;
use crate::error::GreedError;
use crate::platform::account::Account;
use crate::platform::bar::bar_request::BarRequest;
use crate::platform::bars::Bars;
use crate::platform::order::Order;
use crate::platform::position::Position;
use crate::platform::quote::Quote;
use crate::platform::request::OrderRequest;
use crate::platform::FinancialPlatform;
use async_trait::async_trait;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub struct MockPlatform {
    account: Account,
    positions: Vec<Position>,
    open_orders: Vec<Order>,
    recent_orders: Vec<Order>,
    quotes: Vec<Quote>,
    bars: Bars,
    bars_results: Mutex<VecDeque<Result<Bars, String>>>,
    bar_requests: Mutex<Vec<BarRequest>>,
    place_order_result: Order,
    placed_orders: Mutex<Vec<OrderRequest>>,
}

impl Default for MockPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl MockPlatform {
    pub fn new() -> Self {
        Self {
            account: Account::default(),
            positions: Vec::new(),
            open_orders: Vec::new(),
            recent_orders: Vec::new(),
            quotes: Vec::new(),
            bars: Bars::default(),
            bars_results: Mutex::new(VecDeque::new()),
            bar_requests: Mutex::new(Vec::new()),
            place_order_result: Order::default(),
            placed_orders: Mutex::new(Vec::new()),
        }
    }

    pub fn with_account(mut self, account: Account) -> Self {
        self.account = account;
        self
    }

    pub fn with_positions(mut self, positions: Vec<Position>) -> Self {
        self.positions = positions;
        self
    }

    pub fn with_open_orders(mut self, orders: Vec<Order>) -> Self {
        self.open_orders = orders;
        self
    }

    pub fn with_recent_orders(mut self, orders: Vec<Order>) -> Self {
        self.recent_orders = orders;
        self
    }

    pub fn with_quotes(mut self, quotes: Vec<Quote>) -> Self {
        self.quotes = quotes;
        self
    }

    pub fn with_bars(mut self, bars: Bars) -> Self {
        self.bars = bars;
        self
    }

    /// Queues one result per `bars` call, consumed in order; once the queue is
    /// empty, calls fall back to the bars set with `with_bars`.
    pub fn with_bars_results(self, results: Vec<Result<Bars, String>>) -> Self {
        *self.bars_results.lock().unwrap() = results.into();
        self
    }

    pub fn with_place_order_result(mut self, order: Order) -> Self {
        self.place_order_result = order;
        self
    }

    pub fn bar_requests(&self) -> Vec<BarRequest> {
        self.bar_requests.lock().unwrap().clone()
    }

    pub fn placed_orders(&self) -> Vec<OrderRequest> {
        self.placed_orders.lock().unwrap().clone()
    }

    pub fn arc(self) -> Arc<dyn FinancialPlatform> {
        Arc::new(self)
    }
}

#[async_trait]
impl FinancialPlatform for MockPlatform {
    async fn account(&self) -> Result<Account, GreedError> {
        Ok(self.account.clone())
    }

    async fn bars(&self, bars_request: BarRequest) -> Result<Bars, GreedError> {
        self.bar_requests.lock().unwrap().push(bars_request);
        match self.bars_results.lock().unwrap().pop_front() {
            Some(result) => result.map_err(|message| GreedError::new(&message)),
            None => Ok(self.bars.clone()),
        }
    }

    async fn latest_quotes(&self, _symbols: &[AssetSymbol]) -> Result<Vec<Quote>, GreedError> {
        Ok(self.quotes.clone())
    }

    async fn place_order(&self, order_request: OrderRequest) -> Result<Order, GreedError> {
        self.placed_orders.lock().unwrap().push(order_request);
        Ok(self.place_order_result.clone())
    }

    async fn positions(&self) -> Result<Vec<Position>, GreedError> {
        Ok(self.positions.clone())
    }

    async fn open_orders(&self) -> Result<Vec<Order>, GreedError> {
        Ok(self.open_orders.clone())
    }

    async fn recent_orders(&self) -> Result<Vec<Order>, GreedError> {
        Ok(self.recent_orders.clone())
    }
}
