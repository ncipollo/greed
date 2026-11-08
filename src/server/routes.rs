use crate::server::handlers::{
    analyze_handler, init_handler, orders_handler, ping_handler, quote_handler,
    quotes_history_handler, routes_handler, server_info_handler, status_handler,
    status_history_handler,
};
use crate::server::state::AppState;
use axum::routing::{get, post, MethodRouter};
use serde::Serialize;
use std::sync::Arc;

/// Serializable metadata for a single served route, as returned by
/// `GET /routes` and logged at server startup.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RouteInfo {
    pub method: &'static str,
    pub path: &'static str,
    pub description: &'static str,
}

/// Single source of truth for every route the server exposes. The router
/// registers routes by iterating `ServerRoute::all()`, and the `/routes`
/// listing is built from the same table, so the two cannot drift apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerRoute {
    Ping,
    ServerInfo,
    Status,
    Orders,
    Quote,
    Analyze,
    StatusHistory,
    QuotesHistory,
    Init,
    Routes,
}

impl ServerRoute {
    pub fn all() -> &'static [ServerRoute] {
        &[
            ServerRoute::Ping,
            ServerRoute::ServerInfo,
            ServerRoute::Status,
            ServerRoute::Orders,
            ServerRoute::Quote,
            ServerRoute::Analyze,
            ServerRoute::StatusHistory,
            ServerRoute::QuotesHistory,
            ServerRoute::Init,
            ServerRoute::Routes,
        ]
    }

    pub fn path(&self) -> &'static str {
        match self {
            ServerRoute::Ping => "/ping",
            ServerRoute::ServerInfo => "/server/info",
            ServerRoute::Status => "/status",
            ServerRoute::Orders => "/orders",
            ServerRoute::Quote => "/quote",
            ServerRoute::Analyze => "/analyze",
            ServerRoute::StatusHistory => "/history/status",
            ServerRoute::QuotesHistory => "/history/quotes",
            ServerRoute::Init => "/init",
            ServerRoute::Routes => "/routes",
        }
    }

    pub fn method(&self) -> &'static str {
        match self {
            ServerRoute::Init => "POST",
            _ => "GET",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ServerRoute::Ping => "Health check; returns {\"message\":\"pong\"}.",
            ServerRoute::ServerInfo => "Server version, platform type, and simulated flag.",
            ServerRoute::Status => {
                "Portfolio status; optional ?full=true includes positions and orders."
            }
            ServerRoute::Orders => "Recent orders.",
            ServerRoute::Quote => "Latest quotes for comma-separated ?symbols=.",
            ServerRoute::Analyze => "Bars analysis for comma-separated ?symbols=.",
            ServerRoute::StatusHistory => {
                "Portfolio status history; supports from/to/last_hour/hours/days/interval."
            }
            ServerRoute::QuotesHistory => {
                "Stock quote history; supports the same range params plus optional symbols."
            }
            ServerRoute::Init => {
                "Config template generation; returns { filename, content } and never writes to disk."
            }
            ServerRoute::Routes => "Lists every route with method, path, and description.",
        }
    }

    pub fn method_router(&self) -> MethodRouter<Arc<AppState>> {
        match self {
            ServerRoute::Ping => get(ping_handler),
            ServerRoute::ServerInfo => get(server_info_handler),
            ServerRoute::Status => get(status_handler),
            ServerRoute::Orders => get(orders_handler),
            ServerRoute::Quote => get(quote_handler),
            ServerRoute::Analyze => get(analyze_handler),
            ServerRoute::StatusHistory => get(status_history_handler),
            ServerRoute::QuotesHistory => get(quotes_history_handler),
            ServerRoute::Init => post(init_handler),
            ServerRoute::Routes => get(routes_handler),
        }
    }

    pub fn info(&self) -> RouteInfo {
        RouteInfo {
            method: self.method(),
            path: self.path(),
            description: self.description(),
        }
    }
}

pub fn route_infos() -> Vec<RouteInfo> {
    ServerRoute::all().iter().map(ServerRoute::info).collect()
}

/// Human-readable listing of every route, one per line, for startup logs.
pub fn route_listing() -> String {
    let path_width = ServerRoute::all()
        .iter()
        .map(|route| route.path().len())
        .max()
        .unwrap_or(0);
    ServerRoute::all()
        .iter()
        .map(|route| {
            format!(
                "{:<4} {:<path_width$}  {}",
                route.method(),
                route.path(),
                route.description()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod test {
    use crate::server::routes::{route_infos, route_listing, ServerRoute};
    use std::collections::HashSet;

    #[test]
    fn all_contains_expected_method_path_pairs() {
        let pairs: Vec<(&str, &str)> = ServerRoute::all()
            .iter()
            .map(|route| (route.method(), route.path()))
            .collect();
        assert_eq!(
            pairs,
            vec![
                ("GET", "/ping"),
                ("GET", "/server/info"),
                ("GET", "/status"),
                ("GET", "/orders"),
                ("GET", "/quote"),
                ("GET", "/analyze"),
                ("GET", "/history/status"),
                ("GET", "/history/quotes"),
                ("POST", "/init"),
                ("GET", "/routes"),
            ]
        );
    }

    #[test]
    fn no_duplicate_method_path_pairs() {
        let unique: HashSet<(&str, &str)> = ServerRoute::all()
            .iter()
            .map(|route| (route.method(), route.path()))
            .collect();
        assert_eq!(unique.len(), ServerRoute::all().len());
    }

    #[test]
    fn descriptions_are_non_empty() {
        for route in ServerRoute::all() {
            assert!(!route.description().is_empty(), "{route:?}");
        }
    }

    #[test]
    fn route_infos_mirror_all_routes() {
        let infos = route_infos();
        assert_eq!(infos.len(), ServerRoute::all().len());
        for (info, route) in infos.iter().zip(ServerRoute::all()) {
            assert_eq!(*info, route.info());
        }
    }

    #[test]
    fn route_listing_contains_every_route() {
        let listing = route_listing();
        assert_eq!(listing.lines().count(), ServerRoute::all().len());
        for route in ServerRoute::all() {
            assert!(listing.contains(route.path()), "{route:?}");
            assert!(listing.contains(route.description()), "{route:?}");
        }
    }
}
