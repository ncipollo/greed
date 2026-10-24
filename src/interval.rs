use std::str::FromStr;

/// A rollup interval such as "30s", "5m", "1h", or "1d", used to bucket history
/// query results into coarser-grained averages instead of returning raw rows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RollupInterval {
    amount: i64,
    unit: RollupIntervalUnit,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum RollupIntervalUnit {
    Second,
    Minute,
    Hour,
    Day,
}

impl RollupInterval {
    /// Renders this interval as a DuckDB `INTERVAL` literal, e.g. `INTERVAL '5 minutes'`.
    ///
    /// DuckDB has no parameter binding for `INTERVAL` literals, so this is spliced
    /// directly into SQL by callers. That's safe here because `amount` is a plain
    /// `i64` (its `Display` output can't contain quotes or SQL syntax) and `unit`
    /// only ever maps to one of the four fixed words below.
    pub fn to_duckdb_literal(&self) -> String {
        let unit_word = match self.unit {
            RollupIntervalUnit::Second => "seconds",
            RollupIntervalUnit::Minute => "minutes",
            RollupIntervalUnit::Hour => "hours",
            RollupIntervalUnit::Day => "days",
        };
        format!("INTERVAL '{} {}'", self.amount, unit_word)
    }
}

impl FromStr for RollupInterval {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let invalid = || {
            format!(
                "unable to parse '{s}' as a rollup interval; expected a number followed by s, m, h, or d (e.g. '30s', '5m', '1h', '1d')"
            )
        };
        if s.len() < 2 {
            return Err(invalid());
        }
        let (amount_str, unit_str) = s.split_at(s.len() - 1);
        let amount: i64 = amount_str.parse().map_err(|_| invalid())?;
        if amount <= 0 {
            return Err(invalid());
        }
        let unit = match unit_str {
            "s" => RollupIntervalUnit::Second,
            "m" => RollupIntervalUnit::Minute,
            "h" => RollupIntervalUnit::Hour,
            "d" => RollupIntervalUnit::Day,
            _ => return Err(invalid()),
        };
        Ok(Self { amount, unit })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_str_parses_seconds() {
        let interval: RollupInterval = "30s".parse().expect("should parse");
        assert_eq!(interval.to_duckdb_literal(), "INTERVAL '30 seconds'");
    }

    #[test]
    fn from_str_parses_minutes() {
        let interval: RollupInterval = "5m".parse().expect("should parse");
        assert_eq!(interval.to_duckdb_literal(), "INTERVAL '5 minutes'");
    }

    #[test]
    fn from_str_parses_hours() {
        let interval: RollupInterval = "1h".parse().expect("should parse");
        assert_eq!(interval.to_duckdb_literal(), "INTERVAL '1 hours'");
    }

    #[test]
    fn from_str_parses_days() {
        let interval: RollupInterval = "1d".parse().expect("should parse");
        assert_eq!(interval.to_duckdb_literal(), "INTERVAL '1 days'");
    }

    #[test]
    fn from_str_rejects_empty_string() {
        assert!("".parse::<RollupInterval>().is_err());
    }

    #[test]
    fn from_str_rejects_missing_unit() {
        assert!("5".parse::<RollupInterval>().is_err());
    }

    #[test]
    fn from_str_rejects_unknown_unit() {
        assert!("5x".parse::<RollupInterval>().is_err());
    }

    #[test]
    fn from_str_rejects_zero_amount() {
        assert!("0m".parse::<RollupInterval>().is_err());
    }

    #[test]
    fn from_str_rejects_negative_amount() {
        assert!("-5m".parse::<RollupInterval>().is_err());
    }

    #[test]
    fn from_str_rejects_non_numeric_amount() {
        assert!("m5".parse::<RollupInterval>().is_err());
    }

    #[test]
    fn from_str_rejects_garbage() {
        assert!("garbage".parse::<RollupInterval>().is_err());
    }
}
