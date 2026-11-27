use tokio::task_local;

/// Which strategy and tactic is currently running. `TacticRunner::run` sets it for the duration
/// of the tactic, so anything it calls (such as a simulated platform recording a trade) can say
/// why without every signature in between carrying the names.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TacticContext {
    pub strategy: String,
    pub tactic: String,
}

task_local! {
    static CURRENT_TACTIC: TacticContext;
}

/// Runs `future` with `context` as the current tactic.
pub async fn with_tactic<F: std::future::Future>(context: TacticContext, future: F) -> F::Output {
    CURRENT_TACTIC.scope(context, future).await
}

/// The tactic running on this task, if any.
pub fn current_tactic() -> Option<TacticContext> {
    CURRENT_TACTIC.try_with(Clone::clone).ok()
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn the_context_is_visible_inside_and_gone_outside() {
        assert_eq!(current_tactic(), None);
        let context = TacticContext {
            strategy: "core".to_string(),
            tactic: "ETF".to_string(),
        };

        let seen = with_tactic(context.clone(), async { current_tactic() }).await;

        assert_eq!(seen, Some(context));
        assert_eq!(current_tactic(), None);
    }
}
