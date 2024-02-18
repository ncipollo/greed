#[derive(Debug, PartialEq)]
pub enum SkipReason {
    ConditionsUnsatisfied,
    NoTargetAssets,
    Unknown,
}

impl SkipReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            SkipReason::ConditionsUnsatisfied => "when conditions were unsatisfied",
            SkipReason::NoTargetAssets => "no target assets",
            SkipReason::Unknown => "unknown",
        }
    }
}

impl Default for SkipReason {
    fn default() -> Self {
        Self::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_str_conditions_unsatisfied() {
        assert_eq!(
            "when conditions were unsatisfied",
            SkipReason::ConditionsUnsatisfied.as_str()
        )
    }

    #[test]
    fn as_str_no_target_assets() {
        assert_eq!("no target assets", SkipReason::NoTargetAssets.as_str())
    }

    #[test]
    fn as_str_no_unknown() {
        assert_eq!("unknown", SkipReason::Unknown.as_str())
    }

    #[test]
    fn default() {
        assert_eq!(SkipReason::Unknown, Default::default())
    }
}
