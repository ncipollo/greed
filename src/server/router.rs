use crate::server::error::ApiError;
use crate::server::handlers::{
    analyze_handler, orders_handler, ping_handler, quote_handler, quotes_history_handler,
    server_info_handler, status_handler, status_history_handler,
};
use crate::server::state::AppState;
use axum::http::StatusCode;
use axum::routing::get;
use axum::Router;
use std::sync::Arc;

pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/ping", get(ping_handler))
        .route("/server/info", get(server_info_handler))
        .route("/status", get(status_handler))
        .route("/orders", get(orders_handler))
        .route("/quote", get(quote_handler))
        .route("/analyze", get(analyze_handler))
        .route("/history/status", get(status_history_handler))
        .route("/history/quotes", get(quotes_history_handler))
        .fallback(not_found_handler)
        .with_state(state)
}

async fn not_found_handler() -> ApiError {
    ApiError::new(StatusCode::NOT_FOUND, "resource not found")
}
