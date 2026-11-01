use crate::config::platform::PlatformType;
use crate::platform::FinancialPlatform;
use std::path::PathBuf;
use std::sync::Arc;

/// Shared state for HTTP handlers.
pub struct AppState {
    pub platform_type: PlatformType,
    pub is_simulated: bool,
    pub platform: Arc<dyn FinancialPlatform>,
    pub db_path: PathBuf,
}

impl AppState {
    pub fn new(
        platform_type: PlatformType,
        is_simulated: bool,
        platform: Arc<dyn FinancialPlatform>,
        db_path: PathBuf,
    ) -> Self {
        Self {
            platform_type,
            is_simulated,
            platform,
            db_path,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::config::platform::PlatformType;
    use crate::platform::noop::NoOpPlatform;
    use crate::server::state::AppState;
    use std::path::PathBuf;

    #[tokio::test]
    async fn new_sets_fields() {
        let db_path = PathBuf::from("unused.duckdb");
        let state = AppState::new(
            PlatformType::Alpaca,
            true,
            NoOpPlatform::arc(),
            db_path.clone(),
        );
        assert_eq!(state.platform_type, PlatformType::Alpaca);
        assert!(state.is_simulated);
        assert_eq!(state.db_path, db_path);
        assert!(state.platform.account().await.is_ok());
    }
}
