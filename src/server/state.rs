use crate::config::platform::PlatformType;

/// Shared state for HTTP handlers. Currently placeholder-only (platform +
/// run mode); issue #106 will add a handle to the running `GreedRunner`
/// (`crate::run`) and/or the shared DuckDB connection (`crate::db`).
pub struct AppState {
    pub platform: PlatformType,
    pub is_simulated: bool,
}

impl AppState {
    pub fn new(platform: PlatformType, is_simulated: bool) -> Self {
        Self {
            platform,
            is_simulated,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::config::platform::PlatformType;
    use crate::server::state::AppState;

    #[test]
    fn new_sets_fields() {
        let state = AppState::new(PlatformType::Alpaca, true);
        assert_eq!(state.platform, PlatformType::Alpaca);
        assert!(state.is_simulated);
    }
}
