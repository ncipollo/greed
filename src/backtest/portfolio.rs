use crate::asset::AssetSymbol;
use crate::error::GreedError;
use crate::platform::order::Order;
use std::collections::HashMap;

/// Shares of one symbol and what they cost on average.
#[derive(Clone, Debug, PartialEq)]
pub struct Holding {
    pub quantity: f64,
    pub average_cost: f64,
}

/// The simulated account: cash, holdings and every order filled so far.
/// Pure bookkeeping with no clock or market data, so it is easy to test.
#[derive(Debug)]
pub struct Portfolio {
    cash: f64,
    holdings: HashMap<AssetSymbol, Holding>,
    orders: Vec<Order>,
}

impl Portfolio {
    pub fn new(starting_cash: f64) -> Self {
        Self {
            cash: starting_cash,
            holdings: HashMap::new(),
            orders: Vec::new(),
        }
    }

    pub fn cash(&self) -> f64 {
        self.cash
    }

    pub fn holdings(&self) -> &HashMap<AssetSymbol, Holding> {
        &self.holdings
    }

    #[cfg(test)]
    pub fn holding(&self, symbol: &AssetSymbol) -> Option<&Holding> {
        self.holdings.get(symbol)
    }

    pub fn orders(&self) -> &[Order] {
        &self.orders
    }

    pub fn record_order(&mut self, order: Order) {
        self.orders.push(order);
    }

    /// Buys `quantity` shares at `price`. Fails, changing nothing, when cash
    /// can't cover it.
    pub fn buy(
        &mut self,
        symbol: &AssetSymbol,
        quantity: f64,
        price: f64,
    ) -> Result<(), GreedError> {
        let cost = quantity * price;
        if quantity <= 0.0 || price <= 0.0 {
            return Err(GreedError::new(
                "backtest buy needs a positive quantity and price",
            ));
        }
        if cost > self.cash {
            return Err(GreedError::new(&format!(
                "insufficient buying power for {}: need ${cost:.2}, have ${:.2}",
                symbol.symbol, self.cash
            )));
        }
        self.cash -= cost;
        let holding = self.holdings.entry(symbol.clone()).or_insert(Holding {
            quantity: 0.0,
            average_cost: 0.0,
        });
        let total_cost = holding.average_cost * holding.quantity + cost;
        holding.quantity += quantity;
        holding.average_cost = total_cost / holding.quantity;
        Ok(())
    }

    /// Sells `quantity` shares at `price`. Fails, changing nothing, when more
    /// is sold than is held (no shorting).
    pub fn sell(
        &mut self,
        symbol: &AssetSymbol,
        quantity: f64,
        price: f64,
    ) -> Result<(), GreedError> {
        if quantity <= 0.0 || price <= 0.0 {
            return Err(GreedError::new(
                "backtest sell needs a positive quantity and price",
            ));
        }
        let held = self.holdings.get(symbol).map_or(0.0, |h| h.quantity);
        if quantity > held + QUANTITY_TOLERANCE {
            return Err(GreedError::new(&format!(
                "can't sell {quantity} of {}: only {held} held",
                symbol.symbol
            )));
        }
        self.cash += quantity * price;
        if let Some(holding) = self.holdings.get_mut(symbol) {
            holding.quantity -= quantity;
            if holding.quantity <= QUANTITY_TOLERANCE {
                self.holdings.remove(symbol);
            }
        }
        Ok(())
    }

    /// Cash plus holdings marked at `price_of`; holdings without a price are
    /// valued at their cost.
    pub fn equity(&self, price_of: impl Fn(&AssetSymbol) -> Option<f64>) -> f64 {
        self.cash
            + self
                .holdings
                .iter()
                .map(|(symbol, h)| h.quantity * price_of(symbol).unwrap_or(h.average_cost))
                .sum::<f64>()
    }
}

/// Fractional-share arithmetic leaves dust; anything below this counts as zero.
const QUANTITY_TOLERANCE: f64 = 1e-9;

#[cfg(test)]
mod test {
    use super::*;

    fn vti() -> AssetSymbol {
        AssetSymbol::new("VTI")
    }

    #[test]
    fn buy_moves_cash_into_a_holding() {
        let mut portfolio = Portfolio::new(1000.0);

        portfolio.buy(&vti(), 2.0, 100.0).unwrap();

        assert_eq!(portfolio.cash(), 800.0);
        assert_eq!(
            portfolio.holding(&vti()),
            Some(&Holding {
                quantity: 2.0,
                average_cost: 100.0
            })
        );
    }

    #[test]
    fn buying_again_averages_the_cost() {
        let mut portfolio = Portfolio::new(1000.0);

        portfolio.buy(&vti(), 2.0, 100.0).unwrap();
        portfolio.buy(&vti(), 2.0, 200.0).unwrap();

        let holding = portfolio.holding(&vti()).unwrap();
        assert_eq!(holding.quantity, 4.0);
        assert_eq!(holding.average_cost, 150.0);
        assert_eq!(portfolio.cash(), 400.0);
    }

    #[test]
    fn buy_beyond_cash_fails_and_changes_nothing() {
        let mut portfolio = Portfolio::new(100.0);

        let result = portfolio.buy(&vti(), 2.0, 100.0);

        assert!(result.is_err());
        assert_eq!(portfolio.cash(), 100.0);
        assert!(portfolio.holding(&vti()).is_none());
    }

    #[test]
    fn sell_returns_cash_and_reduces_the_holding() {
        let mut portfolio = Portfolio::new(1000.0);
        portfolio.buy(&vti(), 4.0, 100.0).unwrap();

        portfolio.sell(&vti(), 1.0, 150.0).unwrap();

        assert_eq!(portfolio.cash(), 750.0);
        assert_eq!(portfolio.holding(&vti()).unwrap().quantity, 3.0);
        assert_eq!(portfolio.holding(&vti()).unwrap().average_cost, 100.0);
    }

    #[test]
    fn selling_everything_removes_the_holding() {
        let mut portfolio = Portfolio::new(1000.0);
        portfolio.buy(&vti(), 4.0, 100.0).unwrap();

        portfolio.sell(&vti(), 4.0, 100.0).unwrap();

        assert!(portfolio.holding(&vti()).is_none());
        assert_eq!(portfolio.cash(), 1000.0);
    }

    #[test]
    fn selling_more_than_held_fails_and_changes_nothing() {
        let mut portfolio = Portfolio::new(1000.0);
        portfolio.buy(&vti(), 1.0, 100.0).unwrap();

        let result = portfolio.sell(&vti(), 2.0, 100.0);

        assert!(result.is_err());
        assert_eq!(portfolio.cash(), 900.0);
        assert_eq!(portfolio.holding(&vti()).unwrap().quantity, 1.0);
    }

    #[test]
    fn selling_an_unheld_symbol_fails() {
        let mut portfolio = Portfolio::new(1000.0);

        assert!(portfolio.sell(&vti(), 1.0, 100.0).is_err());
    }

    #[test]
    fn fractional_shares_are_supported() {
        let mut portfolio = Portfolio::new(1000.0);

        portfolio.buy(&vti(), 0.5, 100.0).unwrap();
        portfolio.sell(&vti(), 0.5, 100.0).unwrap();

        assert!(portfolio.holding(&vti()).is_none());
    }

    #[test]
    fn equity_marks_holdings_to_market() {
        let mut portfolio = Portfolio::new(1000.0);
        portfolio.buy(&vti(), 2.0, 100.0).unwrap();

        let equity = portfolio.equity(|_| Some(150.0));

        assert_eq!(equity, 800.0 + 300.0);
    }

    #[test]
    fn equity_falls_back_to_cost_without_a_price() {
        let mut portfolio = Portfolio::new(1000.0);
        portfolio.buy(&vti(), 2.0, 100.0).unwrap();

        assert_eq!(portfolio.equity(|_| None), 1000.0);
    }
}
