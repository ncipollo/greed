use crate::asset::AssetSymbol;
use crate::platform::quote::Quote;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Collects quotes fetched during a tick, deduping by symbol so only the
/// latest quote per symbol is kept until it's drained for a single end-of-tick write.
#[derive(Clone, Default)]
pub struct QuoteAccumulator {
    quotes: Arc<Mutex<HashMap<AssetSymbol, Quote>>>,
}

impl QuoteAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&self, quote: Quote) {
        let mut quotes = self.quotes.lock().expect("quote accumulator lock");
        match quotes.get(&quote.symbol) {
            Some(existing) if existing.time > quote.time => {}
            _ => {
                quotes.insert(quote.symbol.clone(), quote);
            }
        }
    }

    pub fn extend(&self, quotes: &[Quote]) {
        for quote in quotes {
            self.add(quote.clone());
        }
    }

    pub fn drain(&self) -> Vec<Quote> {
        let mut quotes = self.quotes.lock().expect("quote accumulator lock");
        quotes.drain().map(|(_, quote)| quote).collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::{TimeZone, Utc};

    fn quote_at(symbol: &str, hour: u32) -> Quote {
        Quote {
            time: Utc.with_ymd_and_hms(2026, 1, 1, hour, 0, 0).unwrap(),
            symbol: AssetSymbol::new(symbol),
            ..Default::default()
        }
    }

    #[test]
    fn add_keeps_a_single_entry_per_symbol() {
        let accumulator = QuoteAccumulator::new();
        accumulator.add(quote_at("VTI", 1));
        accumulator.add(quote_at("VTI", 2));

        assert_eq!(accumulator.drain().len(), 1);
    }

    #[test]
    fn add_keeps_the_newer_quote_for_a_symbol() {
        let accumulator = QuoteAccumulator::new();
        accumulator.add(quote_at("VTI", 1));
        accumulator.add(quote_at("VTI", 2));

        let drained = accumulator.drain();
        assert_eq!(drained[0].time, quote_at("VTI", 2).time);
    }

    #[test]
    fn add_ignores_an_older_quote_for_a_symbol() {
        let accumulator = QuoteAccumulator::new();
        accumulator.add(quote_at("VTI", 2));
        accumulator.add(quote_at("VTI", 1));

        let drained = accumulator.drain();
        assert_eq!(drained[0].time, quote_at("VTI", 2).time);
    }

    #[test]
    fn extend_adds_every_quote_in_the_slice() {
        let accumulator = QuoteAccumulator::new();
        accumulator.extend(&[quote_at("VTI", 1), quote_at("UVXY", 1)]);

        assert_eq!(accumulator.drain().len(), 2);
    }

    #[test]
    fn drain_empties_the_accumulator() {
        let accumulator = QuoteAccumulator::new();
        accumulator.add(quote_at("VTI", 1));
        accumulator.drain();

        assert_eq!(accumulator.drain().len(), 0);
    }
}
