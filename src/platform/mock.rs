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
use std::sync::{Arc, Mutex};

pub struct MockPlatform {
    account: Account,
    positions: Vec<Position>,
    open_orders: Vec<Order>,
    recent_orders: Vec<Order>,
    quotes: Vec<Quote>,
    place_order_result: Order,
    placed_orders: Mutex<Vec<OrderRequest>>,
}

impl MockPlatform {
    pub fn new() -> Self {
        Self {
            account: Account::default(),
            positions: Vec::new(),
            open_orders: Vec::new(),
            recent_orders: Vec::new(),
            quotes: Vec::new(),
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

    pub fn with_place_order_result(mut self, order: Order) -> Self {
        self.place_order_result = order;
        self
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

    async fn bars(&self, _bars_request: BarRequest) -> Result<Bars, GreedError> {
        Ok(Bars::default())
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
