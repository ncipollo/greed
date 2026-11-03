use crate::server::error::ApiError;
use crate::server::handlers::{ping_handler, server_info_handler, status_handler};
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
        .fallback(not_found_handler)
        .with_state(state)
}

async fn not_found_handler() -> ApiError {
    ApiError::new(StatusCode::NOT_FOUND, "resource not found")
}
