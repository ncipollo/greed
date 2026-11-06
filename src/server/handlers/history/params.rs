use crate::asset::AssetSymbol;
use crate::history_range::HistoryRange;
use crate::interval::RollupInterval;
use crate::server::error::ApiError;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::ops::Range;
use std::str::FromStr;

/// Query parameters shared by `/history/status` and `/history/quotes`.
///
/// Axum's `Query<T>` extractor goes through `serde_urlencoded`, which has no
/// support for deserializing a comma-separated or repeated-key list into a
/// `Vec<AssetSymbol>`, and `HistoryRange`/`RollupInterval` only implement
/// `FromStr`, not `Deserialize`. So this struct stays flat (plain
/// `String`/`Option<...>` fields only) and the conversions happen in the
/// methods below -- each mapping a parse failure to `ApiError::bad_request`
/// (400) explicitly, rather than the blanket `From<GreedError> for ApiError`
/// (500): a malformed query string is a client error, not a server one.
#[derive(Debug, Default, Deserialize)]
pub struct HistoryQueryParams {
    /// Start of an explicit range, e.g. "2026-10-15" or "2026-10-15 14:00".
    pub from: Option<String>,
    /// End of an explicit range; defaults to now if omitted.
    pub to: Option<String>,
    /// Shorthand for the last hour.
    #[serde(default)]
    pub last_hour: bool,
    /// Shorthand for the last N hours.
    pub hours: Option<i64>,
    /// Shorthand for the last N days.
    pub days: Option<i64>,
    /// Roll results up into averages per interval (e.g. "30s", "5m", "1h", "1d"); omit for raw rows.
    pub interval: Option<String>,
    /// Comma-separated asset symbols (e.g. `?symbols=VTI,AAPL`), used only by
    /// `/history/quotes`. Chosen over repeated `?symbols=VTI&symbols=AAPL`
    /// keys since `serde_urlencoded` doesn't collect repeated keys into a
    /// `Vec` by default.
    pub symbols: Option<String>,
}

impl HistoryQueryParams {
    /// Resolves the requested time range, mapping an unparseable `from`/`to`
    /// to a 400 instead of the 500 a bare `GreedError` would become.
    pub fn range(&self) -> Result<Range<DateTime<Utc>>, ApiError> {
        let range = HistoryRange {
            from: self.from.clone(),
            to: self.to.clone(),
            last_hour: self.last_hour,
            hours: self.hours,
            days: self.days,
        };
        range
            .resolve(Utc::now())
            .map_err(|err| ApiError::bad_request(err.to_string()))
    }

    /// Parses `interval` into a `RollupInterval`, or `None` when the caller
    /// wants raw rows instead of a rollup.
    pub fn rollup_interval(&self) -> Result<Option<RollupInterval>, ApiError> {
        self.interval
            .as_deref()
            .map(|interval| RollupInterval::from_str(interval).map_err(ApiError::bad_request))
            .transpose()
    }

    /// Splits `symbols` on `,`, trimming whitespace and dropping empty
    /// entries. An absent/empty param means "no filter" (all symbols).
    /// Case-insensitive: each entry is upper-cased to match the canonical
    /// form `AssetSymbol` stores ticker data in (see its `Display` impl).
    pub fn symbols(&self) -> Vec<AssetSymbol> {
        match &self.symbols {
            Some(symbols) => symbols
                .split(',')
                .map(str::trim)
                .filter(|symbol| !symbol.is_empty())
                .map(|symbol| AssetSymbol::from(symbol.to_uppercase()))
                .collect(),
            None => Vec::new(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::Duration;

    #[test]
    fn range_defaults_to_last_day() {
        let params = HistoryQueryParams::default();
        let range = params.range().expect("default range should resolve");
        assert_eq!(range.end - range.start, Duration::days(1));
    }

    #[test]
    fn range_rejects_unparseable_from() {
        let params = HistoryQueryParams {
            from: Some("not-a-date".to_string()),
            ..Default::default()
        };
        assert!(params.range().is_err());
    }

    #[test]
    fn rollup_interval_defaults_to_none() {
        let params = HistoryQueryParams::default();
        assert!(params.rollup_interval().expect("should resolve").is_none());
    }

    #[test]
    fn rollup_interval_parses_valid_value() {
        let params = HistoryQueryParams {
            interval: Some("5m".to_string()),
            ..Default::default()
        };
        assert!(params.rollup_interval().expect("should resolve").is_some());
    }

    #[test]
    fn rollup_interval_rejects_invalid_value() {
        let params = HistoryQueryParams {
            interval: Some("bogus".to_string()),
            ..Default::default()
        };
        assert!(params.rollup_interval().is_err());
    }

    #[test]
    fn symbols_defaults_to_empty_when_absent() {
        let params = HistoryQueryParams::default();
        assert_eq!(params.symbols(), Vec::new());
    }

    #[test]
    fn symbols_splits_and_trims_comma_separated_list() {
        let params = HistoryQueryParams {
            symbols: Some(" VTI, AAPL ,,".to_string()),
            ..Default::default()
        };
        assert_eq!(
            params.symbols(),
            vec![AssetSymbol::new("VTI"), AssetSymbol::new("AAPL")]
        );
    }
}
