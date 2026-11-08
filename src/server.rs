mod error;
mod handlers;
mod router;
mod routes;
mod state;

use crate::error::GreedError;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

pub use error::ApiError;
pub use state::AppState;

/// Default port the `greed server` command binds the HTTP API to when
/// `--port` is omitted. Also shown as the default in `greed server --help`.
pub const DEFAULT_PORT: u16 = 7777;

/// Starts the HTTP server in the background and returns the address it
/// bound to. Binds to an OS-assigned port so concurrent callers (e.g.
/// tests) never collide.
pub async fn start(state: Arc<AppState>) -> Result<SocketAddr, GreedError> {
    let router = router::build_router(state);
    let listener = TcpListener::bind("0.0.0.0:0").await?;
    let addr = listener.local_addr()?;
    log_startup(addr);

    tokio::spawn(async move {
        axum::serve(listener, router).await.ok();
    });

    Ok(addr)
}

/// Runs the HTTP server on `127.0.0.1:<port>` until the listener errors.
/// Intended to run concurrently with the greed loop via `tokio::try_join!`
/// in `greed_server_loop`; does not return under normal operation.
pub async fn serve(state: Arc<AppState>, port: u16) -> Result<(), GreedError> {
    let router = router::build_router(state);
    let listener = TcpListener::bind(format!("127.0.0.1:{port}")).await?;
    log_startup(listener.local_addr()?);
    axum::serve(listener, router).await?;
    Ok(())
}

/// Logs the bound URL and every served route so the API surface is visible
/// as soon as the server starts.
fn log_startup(addr: SocketAddr) {
    log::info!("greed server listening on http://{addr}");
    log::info!("Routes:");
    for line in routes::route_listing().lines() {
        log::info!("  {line}");
    }
}

#[cfg(test)]
mod test {
    use crate::asset::AssetSymbol;
    use crate::config::platform::PlatformType;
    use crate::platform::noop::NoOpPlatform;
    use crate::platform::order::Order;
    use crate::platform::FinancialPlatform;
    use crate::platform::MockPlatform;
    use crate::server::{start, AppState};
    use serde_json::Value;
    use std::path::PathBuf;
    use std::sync::Arc;

    fn test_state(platform_type: PlatformType, is_simulated: bool) -> Arc<AppState> {
        Arc::new(AppState::new(
            platform_type,
            is_simulated,
            NoOpPlatform::arc(),
            PathBuf::from("unused.duckdb"),
        ))
    }

    fn test_state_with_platform(platform: Arc<dyn FinancialPlatform>) -> Arc<AppState> {
        Arc::new(AppState::new(
            PlatformType::Alpaca,
            false,
            platform,
            PathBuf::from("unused.duckdb"),
        ))
    }

    #[tokio::test]
    async fn ping_endpoint_returns_pong() {
        let addr = start(test_state(PlatformType::Alpaca, false))
            .await
            .unwrap();
        let response = reqwest::get(format!("http://{addr}/ping")).await.unwrap();
        assert!(response.status().is_success());
        let body: Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
        assert_eq!(body["message"], "pong");
    }

    #[tokio::test]
    async fn server_info_endpoint_returns_state() {
        let addr = start(test_state(PlatformType::Alpaca, true)).await.unwrap();
        let response = reqwest::get(format!("http://{addr}/server/info"))
            .await
            .unwrap();
        assert!(response.status().is_success());
        let body: Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
        assert_eq!(body["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(body["platform"], "alpaca");
        assert_eq!(body["simulated"], true);
    }

    #[tokio::test]
    async fn status_endpoint_returns_account_without_positions_or_orders_by_default() {
        let addr = start(test_state(PlatformType::Alpaca, false))
            .await
            .unwrap();
        let response = reqwest::get(format!("http://{addr}/status")).await.unwrap();
        assert!(response.status().is_success());
        let body: Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
        assert!(body.get("account").is_some());
        assert!(body.get("positions").is_none());
        assert!(body.get("orders").is_none());
    }

    #[tokio::test]
    async fn status_endpoint_includes_positions_and_orders_when_full() {
        let addr = start(test_state(PlatformType::Alpaca, false))
            .await
            .unwrap();
        let response = reqwest::get(format!("http://{addr}/status?full=true"))
            .await
            .unwrap();
        assert!(response.status().is_success());
        let body: Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
        assert_eq!(body["positions"], serde_json::json!([]));
        assert_eq!(body["orders"], serde_json::json!([]));
    }

    #[tokio::test]
    async fn orders_endpoint_returns_recent_orders() {
        let order = Order::fixture(AssetSymbol::new("vti"));
        let platform = MockPlatform::new().with_recent_orders(vec![order.clone()]);
        let addr = start(test_state_with_platform(platform.arc()))
            .await
            .unwrap();
        let response = reqwest::get(format!("http://{addr}/orders")).await.unwrap();
        assert!(response.status().is_success());
        let body: Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
        let expected = serde_json::to_value(vec![order]).unwrap();
        assert_eq!(body, expected);
    }

    #[tokio::test]
    async fn quote_endpoint_returns_quotes_for_symbols() {
        let addr = start(test_state(PlatformType::Alpaca, false))
            .await
            .unwrap();
        let response = reqwest::get(format!("http://{addr}/quote?symbols=VTI,MSFT"))
            .await
            .unwrap();
        assert!(response.status().is_success());
        let body: Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
        // NoOpPlatform returns no quotes regardless of the symbols requested.
        assert_eq!(body["quotes"], serde_json::json!([]));
    }

    #[tokio::test]
    async fn quote_endpoint_returns_bad_request_when_symbols_missing() {
        let addr = start(test_state(PlatformType::Alpaca, false))
            .await
            .unwrap();
        let response = reqwest::get(format!("http://{addr}/quote")).await.unwrap();
        assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
        let body: Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
        assert!(body["message"].is_string());
    }

    #[tokio::test]
    async fn analyze_endpoint_returns_a_result_per_symbol() {
        let addr = start(test_state(PlatformType::Alpaca, true)).await.unwrap();
        let response = reqwest::get(format!("http://{addr}/analyze?symbols=AAPL,MSFT"))
            .await
            .unwrap();
        assert!(response.status().is_success());
        let body: Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
        let results = body.as_array().unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0]["symbol"], "AAPL");
        assert_eq!(results[1]["symbol"], "MSFT");
    }

    #[tokio::test]
    async fn analyze_endpoint_missing_symbols_returns_bad_request() {
        let addr = start(test_state(PlatformType::Alpaca, true)).await.unwrap();
        let response = reqwest::get(format!("http://{addr}/analyze"))
            .await
            .unwrap();
        assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
        let body: Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
        assert!(body["message"].is_string());
    }

    #[tokio::test]
    async fn init_endpoint_returns_greed_template() {
        let addr = start(test_state(PlatformType::Alpaca, false))
            .await
            .unwrap();
        let response = reqwest::Client::new()
            .post(format!("http://{addr}/init?config_type=greed"))
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let body: Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
        assert_eq!(body["filename"], "greed.toml");
        assert_eq!(body["content"], crate::template::greed_config_template());
    }

    #[tokio::test]
    async fn init_endpoint_unknown_config_type_returns_json_error_body() {
        let addr = start(test_state(PlatformType::Alpaca, false))
            .await
            .unwrap();
        let response = reqwest::Client::new()
            .post(format!("http://{addr}/init?config_type=bogus"))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
        let body: Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
        assert_eq!(body["message"], "unknown config type: bogus");
    }

    #[tokio::test]
    async fn routes_endpoint_lists_every_route() {
        let addr = start(test_state(PlatformType::Alpaca, false))
            .await
            .unwrap();
        let response = reqwest::get(format!("http://{addr}/routes")).await.unwrap();
        assert!(response.status().is_success());
        let body: Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
        let routes = body["routes"].as_array().unwrap();
        let has = |method: &str, path: &str| {
            routes
                .iter()
                .any(|route| route["method"] == method && route["path"] == path)
        };
        assert!(has("GET", "/ping"));
        assert!(has("GET", "/routes"));
        assert!(has("POST", "/init"));
        assert!(routes
            .iter()
            .all(|route| route["description"].as_str().is_some_and(|d| !d.is_empty())));
    }

    #[tokio::test]
    async fn unknown_route_returns_json_error_body() {
        let addr = start(test_state(PlatformType::Alpaca, false))
            .await
            .unwrap();
        let response = reqwest::get(format!("http://{addr}/does-not-exist"))
            .await
            .unwrap();
        assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
        let body: Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
        assert!(body["message"].is_string());
    }
}
