use crate::backtest::trace::BacktestTrace;
use crate::error::GreedError;
use std::path::Path;

/// How a trace file is written, chosen by its extension.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TraceFormat {
    Json,
    Toml,
}

impl TraceFormat {
    /// `.json` or `.toml`, in any case. Checked before a backtest starts so a typo doesn't cost
    /// a whole simulation.
    pub fn from_path(path: &Path) -> Result<Self, GreedError> {
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_ascii_lowercase);
        match extension.as_deref() {
            Some("json") => Ok(Self::Json),
            Some("toml") => Ok(Self::Toml),
            _ => Err(GreedError::new(&format!(
                "can't tell the trace format of '{}': use a .json or .toml extension",
                path.display()
            ))),
        }
    }

    pub fn render(&self, trace: &BacktestTrace) -> Result<String, GreedError> {
        match self {
            Self::Json => serde_json::to_string_pretty(trace).map_err(GreedError::from_display),
            Self::Toml => toml::to_string(trace).map_err(GreedError::from_display),
        }
    }
}

/// Writes `trace` to `path` in the format its extension names. The parent directory must exist.
pub fn write(path: &Path, trace: &BacktestTrace) -> Result<(), GreedError> {
    let format = TraceFormat::from_path(path)?;
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        if !parent.is_dir() {
            return Err(GreedError::new(&format!(
                "can't write the trace: directory '{}' doesn't exist",
                parent.display()
            )));
        }
    }
    let text = format.render(trace)?;
    std::fs::write(path, text).map_err(|e| {
        GreedError::new(&format!(
            "can't write the trace to '{}': {e}",
            path.display()
        ))
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::backtest::trace::fixture;
    use std::path::PathBuf;
    use uuid::Uuid;

    fn temp(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("greed_trace_{}_{name}", Uuid::new_v4()))
    }

    #[test]
    fn format_comes_from_the_extension_in_any_case() {
        assert_eq!(
            TraceFormat::from_path(Path::new("out.json")).unwrap(),
            TraceFormat::Json
        );
        assert_eq!(
            TraceFormat::from_path(Path::new("out.TOML")).unwrap(),
            TraceFormat::Toml
        );
        assert_eq!(
            TraceFormat::from_path(Path::new("dir/out.Json")).unwrap(),
            TraceFormat::Json
        );
    }

    #[test]
    fn unknown_or_missing_extensions_are_rejected() {
        assert!(TraceFormat::from_path(Path::new("out.txt")).is_err());
        assert!(TraceFormat::from_path(Path::new("out")).is_err());
    }

    #[test]
    fn json_round_trips() {
        let trace = fixture::trace();

        let text = TraceFormat::Json.render(&trace).unwrap();

        let parsed: BacktestTrace = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed, trace);
    }

    #[test]
    fn toml_round_trips_even_with_missing_optional_fields() {
        let trace = fixture::trace();

        let text = TraceFormat::Toml.render(&trace).unwrap();

        let parsed: BacktestTrace = toml::from_str(&text).unwrap();
        assert_eq!(parsed, trace);
        assert!(!text.contains("null"));
    }

    #[test]
    fn toml_handles_an_empty_trace() {
        let mut trace = fixture::trace();
        trace.events.clear();
        trace.equity.clear();

        let text = TraceFormat::Toml.render(&trace).unwrap();

        let parsed: BacktestTrace = toml::from_str(&text).unwrap();
        assert_eq!(parsed, trace);
    }

    #[test]
    fn write_creates_the_file() {
        let path = temp("trace.json");

        write(&path, &fixture::trace()).unwrap();

        assert!(std::fs::read_to_string(&path)
            .unwrap()
            .contains("\"summary\""));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn write_fails_clearly_when_the_directory_is_missing() {
        let path = temp("nope").join("trace.json");

        let error = write(&path, &fixture::trace()).unwrap_err().to_string();

        assert!(error.contains("doesn't exist"), "{error}");
    }
}
