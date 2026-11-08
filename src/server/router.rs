use crate::server::error::ApiError;
use crate::server::routes::ServerRoute;
use crate::server::state::AppState;
use axum::http::StatusCode;
use axum::Router;
use std::sync::Arc;

pub fn build_router(state: Arc<AppState>) -> Router {
    ServerRoute::all()
        .iter()
        .fold(Router::new(), |router, route| {
            router.route(route.path(), route.method_router())
        })
        .fallback(not_found_handler)
        .with_state(state)
}

async fn not_found_handler() -> ApiError {
    ApiError::new(StatusCode::NOT_FOUND, "resource not found")
}
