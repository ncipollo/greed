mod error;
mod handlers;
mod router;
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
    axum::serve(listener, router).await?;
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::config::platform::PlatformType;
    use crate::platform::noop::NoOpPlatform;
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
