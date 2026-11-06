use crate::server::error::ApiError;
use crate::template;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

/// Query params for `POST /init`. `config_type` is a plain `Option<String>`
/// (rather than a typed enum) so a missing value comes back as `None`
/// instead of axum producing its own query-rejection response outside of
/// `ApiError` — see `init_handler` below.
#[derive(Debug, Deserialize)]
pub struct InitQueryParams {
    pub config_type: Option<String>,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct InitResponse {
    pub filename: String,
    pub content: String,
}

/// Generates a starter config template, mirroring the `greed init` CLI
/// command's output. `config_type` is accepted as a query param
/// (`POST /init?config_type=greed`) rather than a JSON body, since it's a
/// single string selector and this keeps the endpoint from needing to parse
/// a request body at all.
///
/// Unlike the CLI command, this endpoint never writes to disk: the server
/// may be running on a different host than the caller, so writing the
/// generated file to the *server's* local filesystem would silently do the
/// wrong thing. Callers that want a file on disk should write the returned
/// `content` themselves.
///
/// `config_type` matching is intentionally duplicated from
/// `crate::cli::init::InitConfigType` here as plain string matching: `src/cli`
/// is only declared as a module in the binary crate (`main.rs`), not in
/// `lib.rs`, so this handler (part of the library crate's `server` module)
/// cannot reference that enum directly. If a new config type is ever added,
/// update both `crate::cli::init::InitConfigType` and the match below.
pub async fn init_handler(
    Query(params): Query<InitQueryParams>,
) -> Result<Json<InitResponse>, ApiError> {
    let config_type = params
        .config_type
        .ok_or_else(|| ApiError::new(StatusCode::BAD_REQUEST, "missing config_type"))?;

    let (filename, content) = match config_type.as_str() {
        "greed" => ("greed.toml", template::greed_config_template()),
        "strategy" => ("strategy.toml", template::strategy_config_template()),
        "agent" => ("agent.toml", template::agent_config_template()),
        other => {
            return Err(ApiError::new(
                StatusCode::BAD_REQUEST,
                format!("unknown config type: {other}"),
            ))
        }
    };

    Ok(Json(InitResponse {
        filename: filename.to_string(),
        content: content.to_string(),
    }))
}

#[cfg(test)]
mod test {
    use crate::server::handlers::init::{init_handler, InitQueryParams};
    use crate::template;
    use axum::extract::Query;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use serde_json::Value;

    async fn body_json(response: axum::response::Response) -> Value {
        let (_parts, body) = response.into_parts();
        let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn greed_config_type_returns_greed_template() {
        let response = init_handler(Query(InitQueryParams {
            config_type: Some("greed".to_string()),
        }))
        .await
        .unwrap()
        .0;
        assert_eq!(response.filename, "greed.toml");
        assert_eq!(response.content, template::greed_config_template());
    }

    #[tokio::test]
    async fn strategy_config_type_returns_strategy_template() {
        let response = init_handler(Query(InitQueryParams {
            config_type: Some("strategy".to_string()),
        }))
        .await
        .unwrap()
        .0;
        assert_eq!(response.filename, "strategy.toml");
        assert_eq!(response.content, template::strategy_config_template());
    }

    #[tokio::test]
    async fn agent_config_type_returns_agent_template() {
        let response = init_handler(Query(InitQueryParams {
            config_type: Some("agent".to_string()),
        }))
        .await
        .unwrap()
        .0;
        assert_eq!(response.filename, "agent.toml");
        assert_eq!(response.content, template::agent_config_template());
    }

    #[tokio::test]
    async fn unknown_config_type_returns_bad_request() {
        let error = init_handler(Query(InitQueryParams {
            config_type: Some("bogus".to_string()),
        }))
        .await
        .unwrap_err();
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = body_json(response).await;
        assert_eq!(body["message"], "unknown config type: bogus");
    }

    #[tokio::test]
    async fn missing_config_type_returns_bad_request() {
        let error = init_handler(Query(InitQueryParams { config_type: None }))
            .await
            .unwrap_err();
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = body_json(response).await;
        assert_eq!(body["message"], "missing config_type");
    }
}
