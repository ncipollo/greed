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

#[cfg(test)]
mod test {
    use crate::config::platform::PlatformType;
    use crate::server::{start, AppState};
    use serde_json::Value;
    use std::sync::Arc;

    #[tokio::test]
    async fn ping_endpoint_returns_pong() {
        let addr = start(Arc::new(AppState::new(PlatformType::Alpaca, false)))
            .await
            .unwrap();
        let response = reqwest::get(format!("http://{addr}/ping")).await.unwrap();
        assert!(response.status().is_success());
        let body: Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
        assert_eq!(body["message"], "pong");
    }

    #[tokio::test]
    async fn server_info_endpoint_returns_state() {
        let addr = start(Arc::new(AppState::new(PlatformType::Alpaca, true)))
            .await
            .unwrap();
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
    async fn unknown_route_returns_json_error_body() {
        let addr = start(Arc::new(AppState::new(PlatformType::Alpaca, false)))
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
