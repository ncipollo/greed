use crate::error::GreedError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

/// JSON error response for HTTP handlers. Every non-2xx response from the
/// server should be built from this type, so it always carries a body of
/// the shape `{ "message": "..." }`.
#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
    }
}

#[derive(Serialize)]
struct ErrorBody {
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorBody {
                message: self.message,
            }),
        )
            .into_response()
    }
}

impl From<GreedError> for ApiError {
    fn from(err: GreedError) -> Self {
        ApiError::internal(err.to_string())
    }
}

#[cfg(test)]
mod test {
    use crate::error::GreedError;
    use crate::server::error::ApiError;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use serde_json::Value;

    async fn body_json(response: axum::response::Response) -> Value {
        let (parts, body) = response.into_parts();
        let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        let _ = parts;
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn not_found_has_json_message_body() {
        let response = ApiError::not_found("missing").into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = body_json(response).await;
        assert_eq!(body["message"], "missing");
    }

    #[tokio::test]
    async fn internal_has_json_message_body() {
        let response = ApiError::internal("boom").into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = body_json(response).await;
        assert_eq!(body["message"], "boom");
    }

    #[test]
    fn from_greed_error_maps_to_internal() {
        let api_error = ApiError::from(GreedError::new("boom"));
        assert_eq!(api_error.status, StatusCode::INTERNAL_SERVER_ERROR);
    }
}
