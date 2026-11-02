use crate::platform::order::Order;
use crate::server::error::ApiError;
use crate::server::state::AppState;
use axum::extract::State;
use axum::Json;
use std::sync::Arc;

/// Returns the platform's recent orders as a JSON array, mirroring the
/// `greed orders` CLI command. `OrdersArgs` has no filter/limit flags today
/// (only platform-selection flags, which are already baked into
/// `AppState`), so there are no query params to accept here.
pub async fn orders_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Order>>, ApiError> {
    let orders = state.platform.recent_orders().await?;
    Ok(Json(orders))
}

#[cfg(test)]
mod test {
    use crate::asset::AssetSymbol;
    use crate::config::platform::PlatformType;
    use crate::platform::mock::MockPlatform;
    use crate::platform::order::Order;
    use crate::server::handlers::orders::orders_handler;
    use crate::server::state::AppState;
    use axum::extract::State;
    use std::path::PathBuf;
    use std::sync::Arc;

    #[tokio::test]
    async fn orders_handler_returns_recent_orders() {
        let order = Order::fixture(AssetSymbol::new("vti"));
        let platform = MockPlatform::new().with_recent_orders(vec![order.clone()]);
        let state = Arc::new(AppState::new(
            PlatformType::Alpaca,
            true,
            platform.arc(),
            PathBuf::from("unused.duckdb"),
        ));

        let response = orders_handler(State(state)).await.unwrap().0;
        assert_eq!(response, vec![order]);
    }
}
