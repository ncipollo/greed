use crate::db;
use crate::db::stock_quotes::{select_in_range, select_rollup_in_range};
use crate::matches_symbol_filter;
use crate::server::error::ApiError;
use crate::server::handlers::history::params::HistoryQueryParams;
use crate::server::state::AppState;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::Value;
use std::sync::Arc;

/// `GET /history/quotes` -- stock quote history, filtered by time range and
/// optional comma-separated `symbols`, returned as raw rows or `interval`
/// rollups.
///
/// Reads through `state.db_path` via `db::open_read_only_with_retry` on
/// every request -- the same per-request read-only open (with its existing
/// retry/backoff on lock contention) that `greed history quotes` already
/// uses via `fetch_stock_quote_history` in `src/lib.rs`. `AppState` does not
/// hold a persistent `Connection`; see the PR description for why that's the
/// right call here rather than adding one.
pub async fn quotes_history_handler(
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
            let quotes: Vec<_> = select_in_range(&connection, range)?
                .into_iter()
                .filter(|quote| matches_symbol_filter(&symbols, &quote.symbol))
                .collect();
            Json(quotes).into_response()
        }
    };

    Ok(response)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::asset::AssetSymbol;
    use crate::config::platform::PlatformType;
    use crate::platform::noop::NoOpPlatform;
    use crate::platform::quote::Quote;
    use chrono::Utc;
    use std::path::PathBuf;
    use uuid::Uuid;

    fn temp_db_path() -> PathBuf {
        std::env::temp_dir().join(format!("greed_quotes_history_test_{}.duckdb", Uuid::new_v4()))
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

        let response = quotes_history_handler(State(state), Query(HistoryQueryParams::default()))
            .await
            .expect("handler should succeed");

        assert_eq!(body_json(response).await, serde_json::json!([]));
    }

    #[tokio::test]
    async fn returns_raw_rows_within_range() {
        let path = temp_db_path();
        let connection = db::bootstrap_at(&path).expect("bootstrap should succeed");
        let quote = Quote {
            time: Utc::now(),
            ..Quote::fixture(AssetSymbol::new("VTI"))
        };
        db::stock_quotes::insert(&connection, &quote).expect("insert should succeed");
        drop(connection);

        let response = quotes_history_handler(
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
        let connection = db::bootstrap_at(&path).expect("bootstrap should succeed");
        for symbol in ["VTI", "AAPL"] {
            let quote = Quote {
                time: Utc::now(),
                ..Quote::fixture(AssetSymbol::new(symbol))
            };
            db::stock_quotes::insert(&connection, &quote).expect("insert should succeed");
        }
        drop(connection);

        let params = HistoryQueryParams {
            symbols: Some("aapl".to_string()),
            ..Default::default()
        };
        let response = quotes_history_handler(State(state_for(path.clone())), Query(params))
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
        let connection = db::bootstrap_at(&path).expect("bootstrap should succeed");
        let quote = Quote {
            time: Utc::now(),
            ..Quote::fixture(AssetSymbol::new("VTI"))
        };
        db::stock_quotes::insert(&connection, &quote).expect("insert should succeed");
        drop(connection);

        let params = HistoryQueryParams {
            interval: Some("1h".to_string()),
            ..Default::default()
        };
        let response = quotes_history_handler(State(state_for(path.clone())), Query(params))
            .await
            .expect("handler should succeed");

        let body = body_json(response).await;
        let rollups = body.as_array().expect("should be an array");
        assert_eq!(rollups.len(), 1);
        assert!(rollups[0].get("avg_ask_price").is_some());

        std::fs::remove_file(&path).ok();
    }

    #[tokio::test]
    async fn bad_range_returns_400() {
        let params = HistoryQueryParams {
            from: Some("not-a-date".to_string()),
            ..Default::default()
        };
        let error = quotes_history_handler(State(state_for(temp_db_path())), Query(params))
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
        let error = quotes_history_handler(State(state_for(temp_db_path())), Query(params))
            .await
            .expect_err("handler should fail");

        assert_eq!(
            error.into_response().status(),
            axum::http::StatusCode::BAD_REQUEST
        );
    }
}
