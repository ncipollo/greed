use crate::asset::AssetSymbol;
use crate::fetch_quote_data;
use crate::platform::quote::Quote;
use crate::server::error::ApiError;
use crate::server::state::AppState;
use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Query params for `GET /quote`. `symbols` is a single comma-separated
/// list (e.g. `?symbols=AAPL,MSFT`) rather than a repeated key
/// (`?symbols=AAPL&symbols=MSFT`): axum's `Query` extractor only keeps the
/// last occurrence of a repeated key, and supporting true repetition would
/// require pulling in an additional querystring crate (e.g. `serde_qs`) on
/// top of the `axum`-provided extractor already used elsewhere in this
/// module. A comma-separated value avoids that new dependency.
#[derive(Debug, Deserialize)]
pub struct QuoteQueryParams {
    symbols: Option<String>,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct QuoteResponse {
    pub quotes: Vec<Quote>,
}

/// Splits `raw` on `,`, trims whitespace, and drops empty segments. Returns
/// a 400 `ApiError` if no symbols remain, which covers a missing `symbols`
/// param as well as `?symbols=` / `?symbols=,,`.
fn parse_symbols(raw: Option<String>) -> Result<Vec<AssetSymbol>, ApiError> {
    let symbols: Vec<AssetSymbol> = raw
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(AssetSymbol::from)
        .collect();

    if symbols.is_empty() {
        return Err(ApiError::bad_request("at least one symbol is required"));
    }

    Ok(symbols)
}

pub async fn quote_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<QuoteQueryParams>,
) -> Result<Json<QuoteResponse>, ApiError> {
    let symbols = parse_symbols(params.symbols)?;
    let quotes = fetch_quote_data(&symbols, &state.platform).await?;
    Ok(Json(QuoteResponse { quotes }))
}

#[cfg(test)]
mod test {
    use crate::asset::AssetSymbol;
    use crate::config::platform::PlatformType;
    use crate::platform::noop::NoOpPlatform;
    use crate::platform::quote::Quote;
    use crate::platform::MockPlatform;
    use crate::server::handlers::quote::{quote_handler, QuoteQueryParams, QuoteResponse};
    use crate::server::state::AppState;
    use axum::extract::{Query, State};
    use axum::response::IntoResponse;
    use std::path::PathBuf;
    use std::sync::Arc;

    fn state_with_platform(platform: Arc<dyn crate::platform::FinancialPlatform>) -> Arc<AppState> {
        Arc::new(AppState::new(
            PlatformType::Alpaca,
            false,
            platform,
            PathBuf::from("unused.duckdb"),
        ))
    }

    #[tokio::test]
    async fn quote_handler_returns_quotes_from_platform() {
        let quote = Quote::fixture(AssetSymbol::new("VTI"));
        let platform = MockPlatform::new().with_quotes(vec![quote.clone()]).arc();
        let state = state_with_platform(platform);
        let params = QuoteQueryParams {
            symbols: Some("VTI".to_string()),
        };

        let response = quote_handler(State(state), Query(params))
            .await
            .expect("expected a successful response")
            .0;

        assert_eq!(
            response,
            QuoteResponse {
                quotes: vec![quote]
            }
        );
    }

    #[tokio::test]
    async fn quote_handler_returns_empty_quotes_for_noop_platform() {
        let state = state_with_platform(NoOpPlatform::arc());
        let params = QuoteQueryParams {
            symbols: Some("VTI,MSFT".to_string()),
        };

        let response = quote_handler(State(state), Query(params))
            .await
            .expect("expected a successful response")
            .0;

        assert_eq!(response, QuoteResponse { quotes: vec![] });
    }

    #[tokio::test]
    async fn quote_handler_returns_bad_request_when_symbols_missing() {
        let state = state_with_platform(NoOpPlatform::arc());
        let params = QuoteQueryParams { symbols: None };

        let error = quote_handler(State(state), Query(params))
            .await
            .expect_err("expected a bad request error");

        assert_eq!(
            error.into_response().status(),
            axum::http::StatusCode::BAD_REQUEST
        );
    }

    #[tokio::test]
    async fn quote_handler_returns_bad_request_when_symbols_blank() {
        let state = state_with_platform(NoOpPlatform::arc());
        let params = QuoteQueryParams {
            symbols: Some(",, ,".to_string()),
        };

        let error = quote_handler(State(state), Query(params))
            .await
            .expect_err("expected a bad request error");

        assert_eq!(
            error.into_response().status(),
            axum::http::StatusCode::BAD_REQUEST
        );
    }
}
