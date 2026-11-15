use crate::server::routes::ServerRoute;
use crate::server::DEFAULT_PORT;

pub fn render() -> String {
    let mut out = format!(
        r#"GREED: HTTP SERVER

`greed server <CONFIG> [--port PORT]` runs the trading loop and serves a JSON
HTTP API (default port {DEFAULT_PORT}). It lets other processes observe a live run
without contending for the DuckDB write lock. All endpoints return JSON.

ENDPOINTS
"#
    );
    for route in ServerRoute::all() {
        out.push_str(&format!(
            "  {:<4} {}\n        {}\n",
            route.method(),
            route.path(),
            route.description()
        ));
    }
    out.push_str(DETAILS);
    out
}

const DETAILS: &str = r#"
PARAMETERS (query string)
  /status           full=true|false (default false): include positions and orders
  /quote            symbols=VTI,SPY (required, comma-separated; 400 if empty)
  /analyze          symbols=VTI,SPY (required, comma-separated; 400 if empty)
  /history/status   range: from=2026-10-15 | from=2026-10-15 14:00 [&to=...],
                    or last_hour=true, hours=N, days=N (to defaults to now)
                    interval=30s|5m|1h|1d (optional: average per interval, else raw rows)
  /history/quotes   same as /history/status, plus symbols=VTI,SPY (optional;
                    omitted = all recorded symbols; case-insensitive)
  /init             config_type=greed|strategy|agent (required; no request body)

RESPONSES
  GET /ping          {"message":"pong"}
  GET /server/info   {"version":"...","platform":"alpaca","simulated":false}
  GET /status        portfolio status: account, plus positions/orders when full=true
  GET /orders        JSON array of recent orders
  GET /quote         {"quotes":[...]}
  GET /analyze       JSON array of per-symbol bar analysis
  GET /history/*     JSON array of records (or rollups when interval is set);
                     [] when no history database exists yet
  POST /init         {"filename":"greed.toml","content":"..."}; never writes to disk
  GET /routes        {"routes":[{"method":"GET","path":"/ping","description":"..."}]}

ERRORS
  Every non-2xx response has the JSON body {"message":"..."}:
    400  bad or missing parameter (e.g. {"message":"missing config_type"})
    500  upstream/platform or internal failure

EXAMPLES
  curl localhost:7777/ping
  curl 'localhost:7777/quote?symbols=VTI,SPY'
  curl 'localhost:7777/history/status?hours=6&interval=5m'
  curl -X POST 'localhost:7777/init?config_type=strategy'
"#;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn page_lists_every_route() {
        let page = render();
        for route in ServerRoute::all() {
            let line = format!("{:<4} {}\n", route.method(), route.path());
            assert!(
                page.contains(&line),
                "missing {} {}",
                route.method(),
                route.path()
            );
        }
    }

    #[test]
    fn page_documents_error_shape() {
        assert!(render().contains(r#"{"message":"...""#));
    }
}
