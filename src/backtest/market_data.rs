use crate::asset::AssetSymbol;
use crate::db::stock_bars;
use crate::error::GreedError;
use crate::platform::bar::time_frame::TimeFrame;
use crate::platform::bar::Bar;
use crate::trading_days::TradingDaysOffset;
use chrono::{DateTime, Duration, DurationRound, Utc};
use duckdb::Connection;
use std::collections::HashMap;
use std::ops::Range;

/// Minute bars for every simulated symbol, held in memory so the simulation
/// never touches the database or network.
#[derive(Debug, Default)]
pub struct MarketData {
    minute_bars: HashMap<AssetSymbol, Vec<Bar>>,
}

impl MarketData {
    pub fn new(minute_bars: HashMap<AssetSymbol, Vec<Bar>>) -> Self {
        Self { minute_bars }
    }

    pub fn load(
        connection: &Connection,
        symbols: &[AssetSymbol],
        range: &Range<DateTime<Utc>>,
    ) -> Result<Self, GreedError> {
        let mut minute_bars = HashMap::new();
        for symbol in symbols {
            let bars = stock_bars::load(connection, symbol, &TimeFrame::OneMinute, range)?;
            minute_bars.insert(symbol.clone(), bars);
        }
        Ok(Self { minute_bars })
    }

    pub fn bar_count(&self) -> usize {
        self.minute_bars.values().map(Vec::len).sum()
    }

    /// True when any symbol has a bar in `(now - window, now]`. A longer silence means the market
    /// is closed (overnight, weekend, holiday) rather than one quiet symbol.
    pub fn has_bar_within(&self, now: DateTime<Utc>, window: Duration) -> bool {
        self.minute_bars.values().any(|bars| {
            let upto = bars.partition_point(|bar| bar.timestamp <= now);
            upto.checked_sub(1)
                .and_then(|i| bars.get(i))
                .is_some_and(|bar| bar.timestamp > now - window)
        })
    }

    /// The earliest bar time strictly after `now` across all symbols.
    pub fn next_bar_after(&self, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
        self.minute_bars
            .values()
            .filter_map(|bars| bars.get(bars.partition_point(|bar| bar.timestamp <= now)))
            .map(|bar| bar.timestamp)
            .min()
    }

    /// The newest close at or before `now` however old, for valuing holdings (unlike
    /// `latest_bar`, which refuses stale prices because they're not safe to trade on).
    pub fn last_close(&self, symbol: &AssetSymbol, now: DateTime<Utc>) -> Option<f64> {
        let bars = self.minute_bars.get(symbol)?;
        let upto = bars.partition_point(|bar| bar.timestamp <= now);
        bars.get(upto.checked_sub(1)?).map(|bar| bar.close)
    }

    /// The newest bar at or before `now`, unless it is too old to price from:
    /// older than the previous trading day (so a weekend gap is fine, a
    /// delisted or missing stretch is not).
    pub fn latest_bar(&self, symbol: &AssetSymbol, now: DateTime<Utc>) -> Option<&Bar> {
        let bars = self.minute_bars.get(symbol)?;
        let upto = bars.partition_point(|bar| bar.timestamp <= now);
        let bar = bars.get(upto.checked_sub(1)?)?;
        let oldest_allowed = now.date_naive().previous_trading_day();
        (bar.timestamp.date_naive() >= oldest_allowed).then_some(bar)
    }

    /// Bars with `start <= time <= end`, never later than `now` so a strategy
    /// can't see the future, rolled up to `timeframe`.
    pub fn bars(
        &self,
        symbol: &AssetSymbol,
        timeframe: &TimeFrame,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> Vec<Bar> {
        let end = end.min(now);
        let Some(bars) = self.minute_bars.get(symbol) else {
            return Vec::new();
        };
        let first = bars.partition_point(|bar| bar.timestamp < start);
        let last = bars.partition_point(|bar| bar.timestamp <= end);
        if first >= last {
            return Vec::new();
        }
        aggregate(&bars[first..last], timeframe)
    }
}

fn aggregate(minute_bars: &[Bar], timeframe: &TimeFrame) -> Vec<Bar> {
    let bucket_of = |time: DateTime<Utc>| match timeframe {
        TimeFrame::OneMinute => time,
        TimeFrame::OneHour => time
            .duration_trunc(chrono::Duration::hours(1))
            .unwrap_or(time),
        TimeFrame::OneDay => time
            .duration_trunc(chrono::Duration::days(1))
            .unwrap_or(time),
    };
    let mut rolled: Vec<Bar> = Vec::new();
    for bar in minute_bars {
        let bucket = bucket_of(bar.timestamp);
        match rolled.last_mut() {
            Some(current) if current.timestamp == bucket => {
                current.high = current.high.max(bar.high);
                current.low = current.low.min(bar.low);
                current.close = bar.close;
                current.volume += bar.volume;
            }
            _ => rolled.push(Bar {
                timestamp: bucket,
                ..bar.clone()
            }),
        }
    }
    rolled
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::{Duration, TimeZone};

    fn at(day: u32, hour: u32, minute: u32) -> DateTime<Utc> {
        // 2026-01-05 is a Monday.
        Utc.with_ymd_and_hms(2026, 1, day, hour, minute, 0).unwrap()
    }

    fn bar(time: DateTime<Utc>, open: f64, high: f64, low: f64, close: f64) -> Bar {
        Bar {
            timestamp: time,
            open,
            high,
            low,
            close,
            volume: 10,
        }
    }

    fn vti() -> AssetSymbol {
        AssetSymbol::new("VTI")
    }

    fn data(bars: Vec<Bar>) -> MarketData {
        MarketData::new(HashMap::from([(vti(), bars)]))
    }

    #[test]
    fn latest_bar_is_the_newest_at_or_before_now() {
        let market = data(vec![
            bar(at(5, 14, 30), 1.0, 1.0, 1.0, 1.0),
            bar(at(5, 14, 31), 2.0, 2.0, 2.0, 2.0),
            bar(at(5, 14, 32), 3.0, 3.0, 3.0, 3.0),
        ]);

        let latest = market.latest_bar(&vti(), at(5, 14, 31)).unwrap();

        assert_eq!(latest.close, 2.0);
    }

    #[test]
    fn latest_bar_is_none_before_any_data() {
        let market = data(vec![bar(at(5, 14, 30), 1.0, 1.0, 1.0, 1.0)]);

        assert!(market.latest_bar(&vti(), at(5, 14, 29)).is_none());
    }

    #[test]
    fn latest_bar_is_none_for_an_unknown_symbol() {
        let market = data(vec![]);

        assert!(market
            .latest_bar(&AssetSymbol::new("SPY"), at(5, 14, 30))
            .is_none());
    }

    #[test]
    fn latest_bar_survives_a_weekend_gap() {
        // Friday's last bar, priced on Monday morning.
        let market = data(vec![bar(at(2, 20, 59), 1.0, 1.0, 1.0, 7.0)]);

        let latest = market.latest_bar(&vti(), at(5, 14, 30));

        assert_eq!(latest.unwrap().close, 7.0);
    }

    #[test]
    fn latest_bar_is_omitted_once_stale() {
        // Friday's bar is more than a trading day old by Tuesday.
        let market = data(vec![bar(at(2, 20, 59), 1.0, 1.0, 1.0, 7.0)]);

        assert!(market.latest_bar(&vti(), at(6, 14, 30)).is_none());
    }

    #[test]
    fn bars_never_include_the_future() {
        let market = data(vec![
            bar(at(5, 14, 30), 1.0, 1.0, 1.0, 1.0),
            bar(at(5, 14, 31), 2.0, 2.0, 2.0, 2.0),
            bar(at(5, 14, 32), 3.0, 3.0, 3.0, 3.0),
        ]);

        let bars = market.bars(
            &vti(),
            &TimeFrame::OneMinute,
            at(5, 0, 0),
            at(5, 23, 0),
            at(5, 14, 31),
        );

        let times: Vec<_> = bars.iter().map(|b| b.timestamp).collect();
        assert_eq!(times, vec![at(5, 14, 30), at(5, 14, 31)]);
    }

    #[test]
    fn bars_respect_the_requested_start() {
        let market = data(vec![
            bar(at(5, 14, 30), 1.0, 1.0, 1.0, 1.0),
            bar(at(5, 14, 31), 2.0, 2.0, 2.0, 2.0),
        ]);

        let bars = market.bars(
            &vti(),
            &TimeFrame::OneMinute,
            at(5, 14, 31),
            at(5, 23, 0),
            at(5, 23, 0),
        );

        assert_eq!(bars.len(), 1);
        assert_eq!(bars[0].close, 2.0);
    }

    #[test]
    fn hourly_bars_roll_up_ohlc_and_volume() {
        let market = data(vec![
            bar(at(5, 14, 0), 10.0, 12.0, 9.0, 11.0),
            bar(at(5, 14, 1), 11.0, 15.0, 10.0, 14.0),
            bar(at(5, 14, 59), 14.0, 14.0, 8.0, 9.5),
            bar(at(5, 15, 0), 9.5, 9.6, 9.4, 9.5),
        ]);

        let bars = market.bars(
            &vti(),
            &TimeFrame::OneHour,
            at(5, 0, 0),
            at(5, 23, 0),
            at(5, 23, 0),
        );

        assert_eq!(bars.len(), 2);
        let first = &bars[0];
        assert_eq!(first.timestamp, at(5, 14, 0));
        assert_eq!(
            (first.open, first.high, first.low, first.close, first.volume),
            (10.0, 15.0, 8.0, 9.5, 30)
        );
        assert_eq!(bars[1].timestamp, at(5, 15, 0));
    }

    #[test]
    fn daily_bars_group_by_utc_day() {
        let market = data(vec![
            bar(at(5, 14, 30), 1.0, 2.0, 1.0, 2.0),
            bar(at(5, 15, 30), 2.0, 3.0, 2.0, 3.0),
            bar(at(6, 14, 30), 3.0, 4.0, 3.0, 4.0),
        ]);

        let bars = market.bars(
            &vti(),
            &TimeFrame::OneDay,
            at(5, 0, 0) - Duration::days(1),
            at(7, 0, 0),
            at(7, 0, 0),
        );

        assert_eq!(bars.len(), 2);
        assert_eq!(bars[0].timestamp, at(5, 0, 0));
        assert_eq!((bars[0].open, bars[0].close), (1.0, 3.0));
    }
}
