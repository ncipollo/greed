use crate::platform::status;
use crate::platform::status::PortfolioStatus;
use crate::server::state::AppState;
use crate::server::ApiError;
use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use std::sync::Arc;

/// Query params for `GET /status`. `full` mirrors the CLI's `--full` flag:
/// when true, open positions and orders are included alongside the account.
#[derive(Debug, Default, Deserialize)]
pub struct StatusQuery {
    #[serde(default)]
    pub full: bool,
}

pub async fn status_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<StatusQuery>,
) -> Result<Json<PortfolioStatus>, ApiError> {
    let status = status::fetch(state.platform.as_ref(), query.full).await?;
    Ok(Json(status))
}

#[cfg(test)]
mod test {
    use crate::config::platform::PlatformType;
    use crate::platform::account::Account;
    use crate::platform::noop::NoOpPlatform;
    use crate::server::handlers::status::{status_handler, StatusQuery};
    use crate::server::state::AppState;
    use axum::extract::{Query, State};
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
    async fn not_full_returns_account_without_positions_or_orders() {
        let response = status_handler(State(test_state()), Query(StatusQuery { full: false }))
            .await
            .expect("handler should succeed")
            .0;

        assert_eq!(response.account, Account::default());
        assert_eq!(response.positions, None);
        assert_eq!(response.orders, None);
    }

    #[tokio::test]
    async fn full_returns_positions_and_orders() {
        let response = status_handler(State(test_state()), Query(StatusQuery { full: true }))
            .await
            .expect("handler should succeed")
            .0;

        assert_eq!(response.positions, Some(Vec::new()));
        assert_eq!(response.orders, Some(Vec::new()));
    }
}
