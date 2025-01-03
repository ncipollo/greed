use crate::analysis::result::BarsResult;
use crate::bool::BooleanWhen;
use crate::config::tactic::median::MedianPeriod;
use crate::float::PercentOps;
use crate::platform::bars::Bars;
use crate::tactic::r#for::ForResult;
use crate::tactic::state::TacticState;
use crate::tactic::target::TargetAsset;
use crate::tactic::when::{WhenResult, WhenRule};
use log::{info, warn};

#[derive(Debug, Default, PartialEq)]
pub struct WhenBelowMedianRule {
    below_median_percent: f64,
    median_period: MedianPeriod,
}

impl WhenBelowMedianRule {
    pub fn boxed(below_median_percent: f64, median_period: MedianPeriod) -> Box<dyn WhenRule> {
        Box::new(Self {
            below_median_percent,
            median_period,
        })
    }

    fn is_below_median(&self, state: &TacticState, target_asset: &TargetAsset) -> bool {
        match self.median_period {
            MedianPeriod::Day => self.is_below_median_for_func(state, target_asset, |analysis| {
                &analysis.last_trading_day
            }),
            MedianPeriod::Week => {
                self.is_below_median_for_func(state, target_asset, |analysis| &analysis.seven_day)
            }
            MedianPeriod::Month => {
                self.is_below_median_for_func(state, target_asset, |analysis| &analysis.thirty_day)
            }
        }
    }

    fn is_below_median_for_func<F>(
        &self,
        state: &TacticState,
        target_asset: &TargetAsset,
        func: F,
    ) -> bool
    where
        F: Fn(&BarsResult) -> &Bars,
    {
        if self.is_state_valid(state, target_asset) {
            warn!(
                "when_below_median: state was not valid for: {}",
                target_asset.symbol
            );
            return false;
        }

        let quote = &state.quotes[&target_asset.symbol];
        if !quote.valid_ask() {
            warn!(
                "when_below_median: Ask price is not valid for: {}",
                target_asset.symbol
            );
            return false;
        }

        let analysis = &state.bar_analysis[&target_asset.symbol];
        let median = func(analysis).average_median();
        median
            .filter(Self::is_median_valid)
            .map(|m| {
                let difference_percent = quote.clone().ask_price.percent_below(m);
                (difference_percent >= self.below_median_percent)
                    .when_false(|| self.log_quote_is_not_below_median(difference_percent))
            })
            .unwrap_or(false)
    }

    fn log_quote_is_not_below_median(&self, difference_percent: f64) {
        info!(
            "when_below_median: quote was {:.2} below median, expecting {:.2}.",
            difference_percent, self.below_median_percent
        )
    }

    fn is_state_valid(&self, state: &TacticState, target_asset: &TargetAsset) -> bool {
        !state.quotes.contains_key(&target_asset.symbol)
            || !state.bar_analysis.contains_key(&target_asset.symbol)
    }

    fn is_median_valid(median: &f64) -> bool {
        *median > 0.0
    }
}

impl WhenRule for WhenBelowMedianRule {
    fn evaluate(&self, state: &TacticState, for_result: ForResult) -> WhenResult {
        let assets_below_median = for_result
            .target_assets
            .iter()
            .filter(|t| self.is_below_median(state, t))
            .map(|t| t.clone())
            .collect::<Vec<_>>();
        WhenResult {
            conditions_satisfied: !assets_below_median.is_empty(),
            target_assets: assets_below_median,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::AssetSymbol;
    use crate::platform::quote::Quote;
    use std::collections::HashMap;
    use std::rc::Rc;

    #[test]
    fn evaluate_no_analysis() {
        let state = TacticState {
            bar_analysis: Rc::new(HashMap::new()),
            ..TacticState::fixture()
        };
        let rule = WhenBelowMedianRule::boxed(10.0, MedianPeriod::Day);
        let target_assets = target_assets();
        let for_result = ForResult {
            target_assets: target_assets.clone(),
        };
        let result = rule.evaluate(&state, for_result);
        let expected = WhenResult {
            conditions_satisfied: false,
            target_assets: vec![],
        };
        assert_eq!(expected, result);
    }

    #[test]
    fn evaluate_no_quote() {
        let state = TacticState {
            quotes: HashMap::new(),
            ..TacticState::fixture()
        };
        let rule = WhenBelowMedianRule::boxed(10.0, MedianPeriod::Day);
        let target_assets = target_assets();
        let for_result = ForResult {
            target_assets: target_assets.clone(),
        };
        let result = rule.evaluate(&state, for_result);
        let expected = WhenResult {
            conditions_satisfied: false,
            target_assets: vec![],
        };
        assert_eq!(expected, result);
    }

    #[test]
    fn evaluate_invalid_quote() {
        let spy = AssetSymbol::new("SPY");
        let quote = Quote {
            ask_price: 0.0,
            ..Default::default()
        };
        let state = TacticState {
            quotes: HashMap::from([(spy.clone(), quote)]),
            ..TacticState::fixture()
        };
        let rule = WhenBelowMedianRule::boxed(10.0, MedianPeriod::Day);
        let target_assets = target_assets();
        let for_result = ForResult {
            target_assets: target_assets.clone(),
        };
        let result = rule.evaluate(&state, for_result);
        let expected = WhenResult {
            conditions_satisfied: false,
            target_assets: vec![],
        };
        assert_eq!(expected, result);
    }

    #[test]
    fn evaluate_no_median() {
        let spy = AssetSymbol::new("SPY");
        let bar_result = BarsResult {
            ..Default::default()
        };
        let state = TacticState {
            bar_analysis: Rc::new(HashMap::from([(spy.clone(), bar_result)])),
            ..TacticState::fixture()
        };
        let rule = WhenBelowMedianRule::boxed(10.0, MedianPeriod::Day);
        let target_assets = target_assets();
        let for_result = ForResult {
            target_assets: target_assets.clone(),
        };
        let result = rule.evaluate(&state, for_result);
        let expected = WhenResult {
            conditions_satisfied: false,
            target_assets: vec![],
        };
        assert_eq!(expected, result);
    }

    #[test]
    fn evaluate_zero_median() {
        let spy = AssetSymbol::new("SPY");
        let bar_result = BarsResult {
            last_trading_day: Bars::with_bars(vec![Default::default()]),
            ..Default::default()
        };
        let state = TacticState {
            bar_analysis: Rc::new(HashMap::from([(spy.clone(), bar_result)])),
            ..TacticState::fixture()
        };
        let rule = WhenBelowMedianRule::boxed(10.0, MedianPeriod::Day);
        let target_assets = target_assets();
        let for_result = ForResult {
            target_assets: target_assets.clone(),
        };
        let result = rule.evaluate(&state, for_result);
        let expected = WhenResult {
            conditions_satisfied: false,
            target_assets: vec![],
        };
        assert_eq!(expected, result);
    }

    #[test]
    fn evaluate_1d_satisfied() {
        validate_evaluation(50.0, MedianPeriod::Day, true);
    }

    #[test]
    fn evaluate_1d_not_satisfied() {
        validate_evaluation(51.0, MedianPeriod::Day, false);
    }

    #[test]
    fn evaluate_7d_satisfied() {
        validate_evaluation(33.0, MedianPeriod::Week, true);
    }

    #[test]
    fn evaluate_7d_not_satisfied() {
        validate_evaluation(34.0, MedianPeriod::Week, false);
    }

    #[test]
    fn evaluate_30d_satisfied() {
        validate_evaluation(0.0, MedianPeriod::Month, true);
    }

    #[test]
    fn evaluate_30d_not_satisfied() {
        validate_evaluation(1.0, MedianPeriod::Month, false);
    }

    fn validate_evaluation(
        below_median_percent: f64,
        median_period: MedianPeriod,
        expected_to_be_valid: bool,
    ) {
        let state = TacticState::fixture();
        let rule = WhenBelowMedianRule::boxed(below_median_percent, median_period);
        let target_assets = target_assets();
        let for_result = ForResult {
            target_assets: target_assets.clone(),
        };
        let result = rule.evaluate(&state, for_result);
        let expected_result = if expected_to_be_valid {
            WhenResult {
                conditions_satisfied: true,
                target_assets: target_assets.clone(),
            }
        } else {
            WhenResult {
                conditions_satisfied: false,
                target_assets: vec![],
            }
        };
        assert_eq!(expected_result, result);
    }

    fn target_assets() -> Vec<TargetAsset> {
        vec![
            TargetAsset::full_percent(AssetSymbol::new("SPY")),
            TargetAsset::full_percent(AssetSymbol::new("VTI")),
        ]
    }
}
