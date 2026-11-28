use crate::backtest::trace::BacktestTrace;
use crate::error::GreedError;

const TEMPLATE: &str = include_str!("template.html");
const DATA_MARKER: &str = "/*__GREED_TRACE_DATA__*/";

/// A single self-contained page for `trace`: the data is embedded as JSON and all CSS and JS is
/// inline, so it opens offline with nothing to fetch.
pub fn render(trace: &BacktestTrace) -> Result<String, GreedError> {
    let json = serde_json::to_string(trace).map_err(GreedError::from_display)?;
    Ok(TEMPLATE.replacen(DATA_MARKER, &escape_for_script(&json), 1))
}

/// Makes JSON safe to sit inside a `<script>` element. A `<` could otherwise open `</script>`
/// (or `<!--`) early and let a symbol or tactic name inject markup. The `\uXXXX` forms mean the
/// same thing to JSON, so the data parses back unchanged.
fn escape_for_script(json: &str) -> String {
    json.replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029")
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::backtest::trace::fixture;

    const OPEN: &str = "<script id=\"trace-data\" type=\"application/json\">";

    fn embedded_json(html: &str) -> &str {
        let start = html.find(OPEN).expect("data script") + OPEN.len();
        let end = start + html[start..].find("</script>").expect("closing tag");
        &html[start..end]
    }

    #[test]
    fn the_embedded_data_parses_back_to_the_same_trace() {
        let trace = fixture::trace();

        let html = render(&trace).unwrap();

        let parsed: BacktestTrace = serde_json::from_str(embedded_json(&html)).unwrap();
        assert_eq!(parsed, trace);
    }

    #[test]
    fn markup_in_names_cannot_break_out_of_the_script() {
        let mut trace = fixture::trace();
        trace.events[0].tactic = Some("</script><img src=x onerror=alert(1)>".to_string());
        trace.events[0].symbol = "A&B\u{2028}".to_string();

        let html = render(&trace).unwrap();

        assert!(!html.contains("<img src=x"));
        let data = embedded_json(&html);
        assert!(!data.contains('<') && !data.contains('>') && !data.contains('&'));
        let parsed: BacktestTrace = serde_json::from_str(data).unwrap();
        assert_eq!(parsed, trace);
        // The only closing script tags are the template's own two.
        assert_eq!(
            html.matches("</script>").count(),
            TEMPLATE.matches("</script>").count()
        );
    }

    #[test]
    fn the_page_has_its_skeleton_and_loads_nothing_from_the_network() {
        let html = render(&fixture::trace()).unwrap();

        assert!(html.starts_with("<!doctype html>"));
        for id in [
            "stats",
            "assumptions",
            "assumption-list",
            "unfilled-rows",
            "chart",
            "month",
            "symbol",
            "tactic",
            "search",
            "rows",
        ] {
            assert!(html.contains(&format!("id=\"{id}\"")), "missing #{id}");
        }
        assert!(html.contains("prefers-color-scheme: dark"));
        assert!(!html.contains(DATA_MARKER));
        for forbidden in ["src=\"http", "href=\"http", "@import", "url(http"] {
            assert!(!html.contains(forbidden), "found {forbidden}");
        }
        assert_eq!(html.matches("<script").count(), 2);
    }

    #[test]
    fn the_template_has_exactly_one_data_marker() {
        assert_eq!(TEMPLATE.matches(DATA_MARKER).count(), 1);
    }
}
