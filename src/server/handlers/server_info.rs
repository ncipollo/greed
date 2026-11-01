use crate::config::platform::PlatformType;
use crate::server::state::AppState;
use axum::extract::State;
use axum::Json;
use serde::Serialize;
use std::sync::Arc;

#[derive(Debug, PartialEq, Serialize)]
pub struct ServerInfoResponse {
    pub version: String,
    pub platform: PlatformType,
    pub simulated: bool,
}

pub async fn server_info_handler(State(state): State<Arc<AppState>>) -> Json<ServerInfoResponse> {
    Json(ServerInfoResponse {
        version: env!("CARGO_PKG_VERSION").to_string(),
        platform: state.platform.clone(),
        simulated: state.is_simulated,
    })
}

#[cfg(test)]
mod test {
    use crate::config::platform::PlatformType;
    use crate::server::handlers::server_info::{server_info_handler, ServerInfoResponse};
    use crate::server::state::AppState;
    use axum::extract::State;
    use std::sync::Arc;

    #[tokio::test]
    async fn server_info_echoes_state() {
        let state = Arc::new(AppState::new(PlatformType::Alpaca, true));
        let response = server_info_handler(State(state)).await.0;
        assert_eq!(
            response,
            ServerInfoResponse {
                version: env!("CARGO_PKG_VERSION").to_string(),
                platform: PlatformType::Alpaca,
                simulated: true,
            }
        );
    }
}
