//! ISO 8601 date/datetime parsing shared by `enforce` (format = "datetime")
//! and `query` (date/datetime-aware comparisons, the `now` keyword).

use chrono::{DateTime, Local, NaiveDate, NaiveDateTime};
use std::cmp::Ordering;

/// Human-readable shape accepted by [`parse_iso8601_datetime`], used in error messages.
pub const DATETIME_SHAPE: &str = "YYYY-MM-DDTHH:MM[:SS[.fff]][Z|±HH:MM]";

/// Parse an ISO 8601 datetime: `YYYY-MM-DDTHH:MM`, optional `:SS`, optional
/// fractional seconds (only after `:SS`), optional `Z` or `±HH:MM` offset.
/// A single space is accepted in place of `T`.
///
/// Values with an offset are converted to local time; values without one are
/// taken as local wall-clock time. A bare date (`YYYY-MM-DD`) is rejected.
pub fn parse_iso8601_datetime(s: &str) -> Option<NaiveDateTime> {
    let (body, offset) = split_offset(s)?;
    if !has_datetime_shape(body) {
        return None;
    }
    // chrono's %.f also accepts "no fraction", so one format covers :SS and :SS.fff.
    let normalized = format!("{}T{}", &body[..10], &body[11..]);
    let naive = if body.len() == 16 {
        NaiveDateTime::parse_from_str(&normalized, "%Y-%m-%dT%H:%M").ok()?
    } else {
        NaiveDateTime::parse_from_str(&normalized, "%Y-%m-%dT%H:%M:%S%.f").ok()?
    };
    match offset {
        None => Some(naive),
        Some(off) => {
            let with_offset = format!("{}{}", naive.format("%Y-%m-%dT%H:%M:%S%.f"), off);
            let dt = DateTime::parse_from_str(&with_offset, "%Y-%m-%dT%H:%M:%S%.f%:z").ok()?;
            Some(dt.with_timezone(&Local).naive_local())
        }
    }
}

/// Split a trailing `Z` or `±HH:MM` offset off `s`. Returns the offset in
/// `±HH:MM` form. `None` means the offset part is malformed.
fn split_offset(s: &str) -> Option<(&str, Option<String>)> {
    if let Some(body) = s.strip_suffix('Z') {
        return Some((body, Some("+00:00".to_string())));
    }
    let b = s.as_bytes();
    // An offset can only follow the time part, i.e. at index >= 16.
    if b.len() >= 22 && matches!(b[b.len() - 6], b'+' | b'-') {
        let (body, off) = s.split_at(s.len() - 6);
        let ob = off.as_bytes();
        let ok = ob[1].is_ascii_digit()
            && ob[2].is_ascii_digit()
            && ob[3] == b':'
            && ob[4].is_ascii_digit()
            && ob[5].is_ascii_digit();
        return if ok {
            Some((body, Some(off.to_string())))
        } else {
            None
        };
    }
    Some((s, None))
}

/// Strict shape check for `YYYY-MM-DD(T| )HH:MM[:SS[.f+]]` (no offset).
fn has_datetime_shape(s: &str) -> bool {
    let b = s.as_bytes();
    let digits = |r: std::ops::Range<usize>| b[r].iter().all(u8::is_ascii_digit);
    if b.len() < 16
        || !digits(0..4)
        || b[4] != b'-'
        || !digits(5..7)
        || b[7] != b'-'
        || !digits(8..10)
        || !matches!(b[10], b'T' | b' ')
        || !digits(11..13)
        || b[13] != b':'
        || !digits(14..16)
    {
        return false;
    }
    let rest = &b[16..];
    if rest.is_empty() {
        return true;
    }
    if rest.len() < 3 || rest[0] != b':' || !rest[1..3].iter().all(u8::is_ascii_digit) {
        return false;
    }
    let frac = &rest[3..];
    frac.is_empty()
        || (frac.len() >= 2 && frac[0] == b'.' && frac[1..].iter().all(u8::is_ascii_digit))
}

/// A date (`YYYY-MM-DD`) or an ISO 8601 datetime, for query comparisons.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum When {
    Date(NaiveDate),
    DateTime(NaiveDateTime),
}

impl When {
    pub fn parse(s: &str) -> Option<When> {
        if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
            return Some(When::Date(d));
        }
        parse_iso8601_datetime(s).map(When::DateTime)
    }

    fn date(&self) -> NaiveDate {
        match self {
            When::Date(d) => *d,
            When::DateTime(dt) => dt.date(),
        }
    }

    /// Two datetimes compare to the second; if either side is a bare date,
    /// both are compared by calendar day (so `2026-10-14T09:00 <= 2026-10-14`).
    pub fn compare(&self, other: &When) -> Ordering {
        match (self, other) {
            (When::DateTime(a), When::DateTime(b)) => a.cmp(b),
            _ => self.date().cmp(&other.date()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_datetimes() {
        for s in [
            "2026-10-14T09:00",
            "2026-10-14 09:00",
            "2026-10-14T09:00:30",
            "2026-10-14T09:00:30.5",
            "2026-10-14T09:00:30.123456",
            "2026-10-14T09:00Z",
            "2026-10-14T09:00:30Z",
            "2026-10-14T09:00+05:30",
            "2026-10-14T09:00:30.25-08:00",
            "2026-02-16 18:08:35",
        ] {
            assert!(parse_iso8601_datetime(s).is_some(), "should accept {s:?}");
        }
    }

    #[test]
    fn rejects_invalid_datetimes() {
        for s in [
            "",
            "garbage",
            "2026-10-14",
            "2026-10-14T",
            "2026-10-14T9:00",
            "2026-10-14T09",
            "2026-10-14T09:00:",
            "2026-10-14T09:00:3",
            "2026-10-14T09:00.5",
            "2026-10-14T09:00:30.",
            "2026-10-14T25:00",
            "2026-10-14T09:60",
            "2026-13-01T09:00",
            "2026-02-30T09:00",
            "2026-1-14T09:00",
            "2026/10/14T09:00",
            "2026-10-14t09:00",
            "2026-10-14T09:00+0530",
            "2026-10-14T09:00+5:30",
            "2026-10-14T09:00 +05:30",
            "2026-10-14T09:00z",
            "2026-10-14T09:00:00ZZ",
            "14/10/2026 09:00",
        ] {
            assert!(parse_iso8601_datetime(s).is_none(), "should reject {s:?}");
        }
    }

    #[test]
    fn space_and_t_parse_equal() {
        assert_eq!(
            parse_iso8601_datetime("2026-10-14 09:00"),
            parse_iso8601_datetime("2026-10-14T09:00")
        );
    }

    #[test]
    fn offset_converted_to_local() {
        let utc = parse_iso8601_datetime("2026-10-14T09:00Z").unwrap();
        let plus = parse_iso8601_datetime("2026-10-14T14:30+05:30").unwrap();
        assert_eq!(utc, plus);
    }

    #[test]
    fn when_compare_datetime_vs_datetime() {
        let a = When::parse("2026-10-14T09:00").unwrap();
        let b = When::parse("2026-10-14T09:00:00").unwrap();
        let c = When::parse("2026-10-14 10:00").unwrap();
        assert_eq!(a.compare(&b), Ordering::Equal);
        assert_eq!(a.compare(&c), Ordering::Less);
    }

    #[test]
    fn when_compare_date_vs_datetime_by_day() {
        let day = When::parse("2026-10-14").unwrap();
        let dt = When::parse("2026-10-14T09:00").unwrap();
        let next = When::parse("2026-10-15T00:00").unwrap();
        assert_eq!(dt.compare(&day), Ordering::Equal);
        assert_eq!(next.compare(&day), Ordering::Greater);
    }
}
