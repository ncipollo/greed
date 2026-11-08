use crate::server::routes::{route_infos, RouteInfo};
use axum::Json;
use serde::Serialize;

/// Response for `GET /routes`: `{ "routes": [ { method, path, description } ] }`.
#[derive(Debug, PartialEq, Serialize)]
pub struct RoutesResponse {
    pub routes: Vec<RouteInfo>,
}

pub async fn routes_handler() -> Json<RoutesResponse> {
    Json(RoutesResponse {
        routes: route_infos(),
    })
}

#[cfg(test)]
mod test {
    use crate::server::handlers::routes::routes_handler;
    use crate::server::routes::route_infos;

    #[tokio::test]
    async fn routes_handler_lists_every_route_including_itself() {
        let response = routes_handler().await.0;
        assert_eq!(response.routes, route_infos());
        assert!(response
            .routes
            .iter()
            .any(|route| route.method == "GET" && route.path == "/routes"));
    }
}
