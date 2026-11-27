use chrono::{DateTime, Duration, DurationRound, Utc};
use std::ops::Range;

/// Merges overlapping or touching intervals into a sorted, disjoint list.
/// Empty intervals are dropped.
pub fn merge(mut intervals: Vec<Range<DateTime<Utc>>>) -> Vec<Range<DateTime<Utc>>> {
    intervals.retain(|i| i.start < i.end);
    intervals.sort_by_key(|i| i.start);
    let mut merged: Vec<Range<DateTime<Utc>>> = Vec::new();
    for interval in intervals {
        match merged.last_mut() {
            Some(last) if interval.start <= last.end => last.end = last.end.max(interval.end),
            _ => merged.push(interval),
        }
    }
    merged
}

/// The parts of `range` not covered by `covered`, in order. `covered` may be
/// unsorted and overlapping. Gaps are rounded out to whole minutes.
pub fn subtract(
    range: &Range<DateTime<Utc>>,
    covered: Vec<Range<DateTime<Utc>>>,
) -> Vec<Range<DateTime<Utc>>> {
    let mut gaps = Vec::new();
    let mut cursor = range.start;
    for interval in merge(covered) {
        if interval.end <= cursor {
            continue;
        }
        if interval.start >= range.end {
            break;
        }
        if interval.start > cursor {
            gaps.push(cursor..interval.start);
        }
        cursor = cursor.max(interval.end);
    }
    if cursor < range.end {
        gaps.push(cursor..range.end);
    }
    merge(gaps.into_iter().map(round_to_minutes).collect())
}

fn round_to_minutes(gap: Range<DateTime<Utc>>) -> Range<DateTime<Utc>> {
    let minute = Duration::minutes(1);
    let start = gap.start.duration_trunc(minute).unwrap_or(gap.start);
    let truncated_end = gap.end.duration_trunc(minute).unwrap_or(gap.end);
    let end = if truncated_end < gap.end {
        truncated_end + minute
    } else {
        truncated_end
    };
    start..end
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::TimeZone;

    fn at(day: u32, hour: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, day, hour, 0, 0).unwrap()
    }

    #[test]
    fn merge_combines_overlapping_and_adjacent() {
        let merged = merge(vec![
            at(3, 0)..at(4, 0),
            at(1, 0)..at(2, 0),
            at(2, 0)..at(3, 0),
        ]);

        assert_eq!(merged, vec![at(1, 0)..at(4, 0)]);
    }

    #[test]
    fn merge_keeps_disjoint_intervals_sorted() {
        let merged = merge(vec![at(5, 0)..at(6, 0), at(1, 0)..at(2, 0)]);

        assert_eq!(merged, vec![at(1, 0)..at(2, 0), at(5, 0)..at(6, 0)]);
    }

    #[test]
    fn merge_drops_empty_intervals() {
        assert!(merge(vec![at(1, 0)..at(1, 0)]).is_empty());
    }

    #[test]
    fn subtract_with_no_coverage_returns_whole_range() {
        let gaps = subtract(&(at(1, 0)..at(5, 0)), vec![]);

        assert_eq!(gaps, vec![at(1, 0)..at(5, 0)]);
    }

    #[test]
    fn subtract_with_full_coverage_returns_nothing() {
        let gaps = subtract(&(at(1, 0)..at(5, 0)), vec![at(1, 0)..at(5, 0)]);

        assert!(gaps.is_empty());
    }

    #[test]
    fn subtract_finds_hole_in_the_middle() {
        let gaps = subtract(
            &(at(1, 0)..at(5, 0)),
            vec![at(1, 0)..at(2, 0), at(3, 0)..at(5, 0)],
        );

        assert_eq!(gaps, vec![at(2, 0)..at(3, 0)]);
    }

    #[test]
    fn subtract_handles_coverage_overlapping_the_edges() {
        let gaps = subtract(
            &(at(2, 0)..at(4, 0)),
            vec![at(1, 0)..at(2, 12), at(3, 12)..at(6, 0)],
        );

        assert_eq!(gaps, vec![at(2, 12)..at(3, 12)]);
    }

    #[test]
    fn subtract_merges_adjacent_coverage() {
        let gaps = subtract(
            &(at(1, 0)..at(4, 0)),
            vec![at(1, 0)..at(2, 0), at(2, 0)..at(4, 0)],
        );

        assert!(gaps.is_empty());
    }

    #[test]
    fn subtract_rounds_gaps_out_to_whole_minutes() {
        let start = at(1, 0) + Duration::seconds(30);
        let end = at(1, 1) + Duration::seconds(10);

        let gaps = subtract(&(start..end), vec![]);

        assert_eq!(gaps, vec![at(1, 0)..at(1, 1) + Duration::minutes(1)]);
    }
}
