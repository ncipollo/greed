# Greed

## After Each Change
Run the following commands after every code change and fix any issues before considering the change complete:

1. `cargo fmt` - Format all code
2. `cargo test` - Run all tests
3. `cargo clippy` - Run linter; fix all warnings and errors before completing the change

## Dependencies
Always use exact versions for dependencies in `Cargo.toml` (e.g., `"4.5.60"` not `"4"`). Check `Cargo.lock` for the resolved version when pinning.

## Module Conventions
Never use `mod.rs`. Always use the modern Rust style: create a top-level file (e.g., `foo.rs`) as the module root, and a matching folder (`foo/`) for any submodules.

## Architecture
CLI commands (`src/cli/`) and HTTP server endpoints (`src/server/`) together form greed's "view" layer: both are thin entry points that call into the same core library (`src/run.rs`, `src/platform/`, `src/db/`) and just render the result differently (stdout vs. JSON). Every non-loop CLI command should eventually have a matching HTTP endpoint exposing the same data (e.g. `quote` ↔ `/quote`, `status` ↔ `/status`, `orders` ↔ `/orders`). The one exception is `run`, which blocks forever in `GreedRunner::run_loop` — there is no 1:1 HTTP equivalent for a blocking loop; instead, the server exists so other processes can observe/interact with greed while a run holds the DuckDB write lock, rather than contending with it via a second CLI invocation.

**Exception: `info`.** `greed info [TOPIC]` (`src/cli/info.rs` over the registry in `src/info.rs` + `src/info/`) is static agent documentation and deliberately has no HTTP twin. If wanted later, that is a separate issue (`GET /info`, `GET /info/{topic}`). When you change config formats (config/strategy/tactic/agent keys) or HTTP endpoints, update the relevant `info` topic page; tests parse the embedded examples against the real config types and check every `ServerRoute` appears in the server topic.

`src/server.rs` + `src/server/` is the axum HTTP server. `AppState` (held behind `Arc`) is the shared handler state; it will grow a handle to the DuckDB connection (`src/db.rs`) and the running `GreedRunner` (`src/run.rs`) once the server is wired into a live run. Every non-2xx response must go through `src/server/error.rs`'s `ApiError`, which guarantees a JSON body of the shape `{ "message": "..." }` — reuse it for every new endpoint rather than inventing a new error shape.
