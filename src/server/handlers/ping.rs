use axum::Json;
use serde::Serialize;

#[derive(Debug, PartialEq, Serialize)]
pub struct PingResponse {
    pub message: String,
}

pub async fn ping_handler() -> Json<PingResponse> {
    Json(PingResponse {
        message: "pong".to_string(),
    })
}

#[cfg(test)]
mod test {
    use crate::server::handlers::ping::{ping_handler, PingResponse};

    #[tokio::test]
    async fn ping_returns_pong() {
        let response = ping_handler().await.0;
        assert_eq!(
            response,
            PingResponse {
                message: "pong".to_string()
            }
        );
    }
}
