use chrono::{DateTime, Datelike, NaiveDate, Utc};
use std::ops::Range;

/// Splits `range` at UTC calendar-month boundaries.
///
/// Backfill fetches and saves one chunk at a time, so progress is reportable,
/// memory stays bounded, and an interrupted run loses at most one chunk.
pub fn month_chunks(range: &Range<DateTime<Utc>>) -> Vec<Range<DateTime<Utc>>> {
    let mut chunks = Vec::new();
    let mut start = range.start;
    while start < range.end {
        let end = next_month_start(start).min(range.end);
        chunks.push(start..end);
        start = end;
    }
    chunks
}

fn next_month_start(moment: DateTime<Utc>) -> DateTime<Utc> {
    let date = moment.date_naive();
    let (year, month) = if date.month() == 12 {
        (date.year() + 1, 1)
    } else {
        (date.year(), date.month() + 1)
    };
    NaiveDate::from_ymd_opt(year, month, 1)
        .expect("the first of a month is always valid")
        .and_hms_opt(0, 0, 0)
        .expect("midnight is always valid")
        .and_utc()
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::TimeZone;

    fn at(year: i32, month: u32, day: u32, hour: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(year, month, day, hour, 0, 0).unwrap()
    }

    #[test]
    fn range_within_one_month_is_a_single_chunk() {
        let range = at(2026, 3, 5, 9)..at(2026, 3, 20, 17);
        assert_eq!(month_chunks(&range), vec![range]);
    }

    #[test]
    fn range_spanning_months_splits_at_month_starts() {
        let range = at(2026, 1, 15, 12)..at(2026, 3, 10, 6);
        assert_eq!(
            month_chunks(&range),
            vec![
                at(2026, 1, 15, 12)..at(2026, 2, 1, 0),
                at(2026, 2, 1, 0)..at(2026, 3, 1, 0),
                at(2026, 3, 1, 0)..at(2026, 3, 10, 6),
            ]
        );
    }

    #[test]
    fn range_ending_exactly_on_a_month_start_has_no_empty_chunk() {
        let range = at(2026, 1, 1, 0)..at(2026, 2, 1, 0);
        assert_eq!(month_chunks(&range), vec![range]);
    }

    #[test]
    fn range_crossing_a_year_boundary_rolls_the_year() {
        let range = at(2025, 12, 20, 0)..at(2026, 1, 5, 0);
        assert_eq!(
            month_chunks(&range),
            vec![
                at(2025, 12, 20, 0)..at(2026, 1, 1, 0),
                at(2026, 1, 1, 0)..at(2026, 1, 5, 0),
            ]
        );
    }

    #[test]
    fn a_full_year_is_twelve_or_thirteen_chunks_that_tile_the_range() {
        let range = at(2025, 10, 17, 12)..at(2026, 10, 17, 12);
        let chunks = month_chunks(&range);

        assert_eq!(chunks.len(), 13);
        assert_eq!(chunks.first().unwrap().start, range.start);
        assert_eq!(chunks.last().unwrap().end, range.end);
        assert!(chunks.windows(2).all(|pair| pair[0].end == pair[1].start));
    }

    #[test]
    fn empty_range_has_no_chunks() {
        let range = at(2026, 1, 1, 0)..at(2026, 1, 1, 0);
        assert!(month_chunks(&range).is_empty());
    }
}
