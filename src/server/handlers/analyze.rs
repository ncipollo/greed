use crate::analysis::result::BarsResult;
use crate::analysis::AssetAnalyzer;
use crate::asset::AssetSymbol;
use crate::server::error::ApiError;
use crate::server::state::AppState;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use std::sync::Arc;

/// Query params for `GET /analyze`.
///
/// `symbols` is a single comma-separated query param (e.g.
/// `?symbols=AAPL,MSFT`) rather than a repeated key (`?symbols=AAPL&symbols=MSFT`):
/// axum's `Query` extractor goes through `serde_urlencoded`, which doesn't
/// support collecting repeated keys into a `Vec`, and pulling in a crate like
/// `serde_qs` just for this isn't warranted here.
#[derive(Debug, Deserialize)]
pub struct AnalyzeQuery {
    pub symbols: Option<String>,
}

/// `GET /analyze` — runs the same bars analysis as `greed analyze` for the
/// requested symbols, using the server's already-configured platform (the
/// platform/simulated flag is server configuration, not a request input).
pub async fn analyze_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AnalyzeQuery>,
) -> Result<Json<Vec<BarsResult>>, ApiError> {
    let symbols = parse_symbols(query.symbols)?;
    let analyzer = AssetAnalyzer::new(state.platform.clone());
    let bars_by_symbol = analyzer.analyze_bars(&symbols).await?;
    let results = symbols
        .iter()
        .filter_map(|symbol| bars_by_symbol.get(symbol).cloned())
        .collect();
    Ok(Json(results))
}

fn parse_symbols(symbols: Option<String>) -> Result<Vec<AssetSymbol>, ApiError> {
    let symbols = symbols.unwrap_or_default();
    let parsed: Vec<AssetSymbol> = symbols
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(AssetSymbol::from)
        .collect();
    if parsed.is_empty() {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "symbols query param is required",
        ));
    }
    Ok(parsed)
}

#[cfg(test)]
mod test {
    use crate::config::platform::PlatformType;
    use crate::platform::noop::NoOpPlatform;
    use crate::server::handlers::analyze::{analyze_handler, AnalyzeQuery};
    use crate::server::state::AppState;
    use axum::extract::{Query, State};
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use std::path::PathBuf;
    use std::sync::Arc;

    fn test_state() -> Arc<AppState> {
        Arc::new(AppState::new(
            PlatformType::Alpaca,
            true,
            NoOpPlatform::arc(),
            PathBuf::from("unused.duckdb"),
        ))
    }

    #[tokio::test]
    async fn analyze_returns_a_result_per_symbol() {
        let query = AnalyzeQuery {
            symbols: Some("AAPL,MSFT".to_string()),
        };
        let response = analyze_handler(State(test_state()), Query(query))
            .await
            .expect("expected a successful response")
            .0;
        assert_eq!(response.len(), 2);
        assert_eq!(response[0].symbol.symbol, "AAPL");
        assert_eq!(response[1].symbol.symbol, "MSFT");
    }

    #[tokio::test]
    async fn analyze_missing_symbols_is_bad_request() {
        let query = AnalyzeQuery { symbols: None };
        let error = analyze_handler(State(test_state()), Query(query))
            .await
            .expect_err("expected a bad request error");
        assert_eq!(error.into_response().status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn analyze_empty_symbols_is_bad_request() {
        let query = AnalyzeQuery {
            symbols: Some("".to_string()),
        };
        let error = analyze_handler(State(test_state()), Query(query))
            .await
            .expect_err("expected a bad request error");
        assert_eq!(error.into_response().status(), StatusCode::BAD_REQUEST);
    }
}
