use crate::strategy::action::Action;
use crate::strategy::r#do::{DoResult, DoRule};
use crate::strategy::state::StrategyState;
use crate::strategy::when::WhenResult;

pub struct DoSellAllRule;

impl DoSellAllRule {
    pub fn boxed() -> Box<dyn DoRule> {
        Box::new(Self {})
    }
}

impl DoRule for DoSellAllRule {
    fn evaluate(&self, state: &StrategyState, when_result: WhenResult) -> DoResult {
        let actions = when_result
            .target_assets
            .into_iter()
            .map(|asset| {
                let symbol = asset.symbol;
                let amount = state.positions[&symbol].quantity.clone();
                Action::sell_notional(symbol.clone(), amount)
            })
            .collect::<Vec<_>>();
        DoResult {
            actions,
            ..Default::default()
        }
    }
}
