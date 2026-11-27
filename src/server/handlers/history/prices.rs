use crate::db;
use crate::db::stock_bars::{select_in_range, select_rollup_in_range};
use crate::matches_symbol_filter;
use crate::server::error::ApiError;
use crate::server::handlers::history::params::HistoryQueryParams;
use crate::server::state::AppState;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::Value;
use std::sync::Arc;

/// `GET /history/prices` -- price history (the recorded minute bars), filtered by time
/// range and optional comma-separated `symbols`, returned as raw bars or `interval`
/// OHLCV rollups.
///
/// Reads through `state.db_path` via `db::open_read_only_with_retry` on
/// every request -- the same per-request read-only open (with its existing
/// retry/backoff on lock contention) that `greed history prices` already
/// uses via `fetch_price_history` in `src/lib.rs`. `AppState` does not
/// hold a persistent `Connection`; see the PR description for why that's the
/// right call here rather than adding one.
pub async fn prices_history_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HistoryQueryParams>,
) -> Result<Response, ApiError> {
    let range = params.range()?;
    let interval = params.rollup_interval()?;
    let symbols = params.symbols();

    let connection = db::open_read_only_with_retry(&state.db_path).await?;
    let response = match (connection, interval) {
        // No DB file yet (e.g. `--record-history` was never enabled) --
        // mirrors the CLI, which treats this as "no history" rather than an
        // error.
        (None, _) => Json(Vec::<Value>::new()).into_response(),
        (Some(connection), Some(interval)) => {
            let rollups: Vec<_> = select_rollup_in_range(&connection, range, &interval)?
                .into_iter()
                .filter(|rollup| matches_symbol_filter(&symbols, &rollup.symbol))
                .collect();
            Json(rollups).into_response()
        }
        (Some(connection), None) => {
            let bars: Vec<_> = select_in_range(&connection, range)?
                .into_iter()
                .filter(|bar| matches_symbol_filter(&symbols, &bar.symbol))
                .collect();
            Json(bars).into_response()
        }
    };

    Ok(response)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::asset::AssetSymbol;
    use crate::config::platform::PlatformType;
    use crate::platform::bar::time_frame::TimeFrame;
    use crate::platform::bar::Bar;
    use crate::platform::noop::NoOpPlatform;
    use chrono::{Duration, Utc};
    use std::path::PathBuf;
    use uuid::Uuid;

    fn temp_db_path() -> PathBuf {
        std::env::temp_dir().join(format!(
            "greed_prices_history_test_{}.duckdb",
            Uuid::new_v4()
        ))
    }

    /// Stores one minute bar for `symbol` a minute ago, so it falls in the default range.
    fn store_bar(path: &std::path::Path, symbol: &str) {
        let mut connection = db::bootstrap_at(path).expect("bootstrap should succeed");
        let time = Utc::now() - Duration::minutes(1);
        let bar = Bar {
            timestamp: time,
            open: 1.0,
            high: 2.0,
            low: 0.5,
            close: 1.5,
            volume: 10,
        };
        db::stock_bars::insert_chunk(
            &mut connection,
            &AssetSymbol::new(symbol),
            &TimeFrame::OneMinute,
            &((time - Duration::minutes(1))..(time + Duration::minutes(1))),
            &[bar],
        )
        .expect("insert should succeed");
    }

    fn state_for(db_path: PathBuf) -> Arc<AppState> {
        Arc::new(AppState::new(
            PlatformType::Alpaca,
            true,
            NoOpPlatform::arc(),
            db_path,
        ))
    }

    async fn body_json(response: Response) -> Value {
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn missing_db_returns_empty_array() {
        let state = state_for(temp_db_path());

        let response = prices_history_handler(State(state), Query(HistoryQueryParams::default()))
            .await
            .expect("handler should succeed");

        assert_eq!(body_json(response).await, serde_json::json!([]));
    }

    #[tokio::test]
    async fn returns_raw_rows_within_range() {
        let path = temp_db_path();
        store_bar(&path, "VTI");

        let response = prices_history_handler(
            State(state_for(path.clone())),
            Query(HistoryQueryParams::default()),
        )
        .await
        .expect("handler should succeed");

        let body = body_json(response).await;
        assert_eq!(body.as_array().expect("should be an array").len(), 1);
        assert_eq!(body[0]["symbol"], "VTI");

        std::fs::remove_file(&path).ok();
    }

    #[tokio::test]
    async fn filters_by_symbols() {
        let path = temp_db_path();
        for symbol in ["VTI", "AAPL"] {
            store_bar(&path, symbol);
        }

        let params = HistoryQueryParams {
            symbols: Some("aapl".to_string()),
            ..Default::default()
        };
        let response = prices_history_handler(State(state_for(path.clone())), Query(params))
            .await
            .expect("handler should succeed");

        let body = body_json(response).await;
        assert_eq!(body.as_array().expect("should be an array").len(), 1);
        assert_eq!(body[0]["symbol"], "AAPL");

        std::fs::remove_file(&path).ok();
    }

    #[tokio::test]
    async fn returns_rollups_when_interval_given() {
        let path = temp_db_path();
        store_bar(&path, "VTI");

        let params = HistoryQueryParams {
            interval: Some("1h".to_string()),
            ..Default::default()
        };
        let response = prices_history_handler(State(state_for(path.clone())), Query(params))
            .await
            .expect("handler should succeed");

        let body = body_json(response).await;
        let rollups = body.as_array().expect("should be an array");
        assert_eq!(rollups.len(), 1);
        assert!(rollups[0].get("close").is_some());
        assert!(rollups[0].get("avg_ask_price").is_none());

        std::fs::remove_file(&path).ok();
    }

    #[tokio::test]
    async fn bad_range_returns_400() {
        let params = HistoryQueryParams {
            from: Some("not-a-date".to_string()),
            ..Default::default()
        };
        let error = prices_history_handler(State(state_for(temp_db_path())), Query(params))
            .await
            .expect_err("handler should fail");

        assert_eq!(
            error.into_response().status(),
            axum::http::StatusCode::BAD_REQUEST
        );
    }

    #[tokio::test]
    async fn bad_interval_returns_400() {
        let params = HistoryQueryParams {
            interval: Some("bogus".to_string()),
            ..Default::default()
        };
        let error = prices_history_handler(State(state_for(temp_db_path())), Query(params))
            .await
            .expect_err("handler should fail");

        assert_eq!(
            error.into_response().status(),
            axum::http::StatusCode::BAD_REQUEST
        );
    }
}
