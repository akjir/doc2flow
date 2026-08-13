//! Time and date formatting utilities.

use std::fmt::Write;
use std::time::SystemTime;

/// Formats a [`SystemTime`] as an ISO 8601 UTC timestamp string.
///
/// Returns a string in the format `YYYY-MM-DDTHH:MM:SSZ`. Sub-second precision is omitted.
/// If the timestamp is prior to the Unix epoch, returns `"1970-01-01T00:00:00Z"`.
///
/// # Examples
///
/// ```
/// use std::time::{Duration, UNIX_EPOCH};
/// use doc2flow::core::utils::time::format_iso8601_utc;
///
/// let epoch = UNIX_EPOCH;
/// assert_eq!(format_iso8601_utc(epoch), "1970-01-01T00:00:00Z");
///
/// let timestamp = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
/// assert_eq!(format_iso8601_utc(timestamp), "2023-11-14T22:13:20Z");
/// ```
pub fn format_iso8601_utc(time: SystemTime) -> String {
    let mut out = String::with_capacity(20);
    format_iso8601_utc_into(time, &mut out);
    out
}

/// Formats a [`SystemTime`] as an ISO 8601 UTC timestamp into a buffer.
///
/// Appends a string in the format `YYYY-MM-DDTHH:MM:SSZ` directly into `out`.
/// If the timestamp is prior to the Unix epoch, appends `"1970-01-01T00:00:00Z"`.
///
/// # Examples
///
/// ```
/// use std::time::UNIX_EPOCH;
/// use doc2flow::core::utils::time::format_iso8601_utc_into;
///
/// let mut buf = String::new();
/// format_iso8601_utc_into(UNIX_EPOCH, &mut buf);
/// assert_eq!(buf, "1970-01-01T00:00:00Z");
/// ```
pub fn format_iso8601_utc_into(time: SystemTime, out: &mut String) {
    let dur = time
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();

    let sec = secs % 60;
    let mins = secs / 60;
    let min = mins % 60;
    let hours = mins / 60;
    let hour = hours % 24;
    let days = hours / 24;

    let z = days as i64 + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = y + if m <= 2 { 1 } else { 0 };

    out.reserve(20);
    let _ = write!(out, "{year:04}-{m:02}-{d:02}T{hour:02}:{min:02}:{sec:02}Z");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_iso8601_utc_current_time_length() {
        let now = std::time::SystemTime::now();
        let formatted = format_iso8601_utc(now);
        assert!(formatted.ends_with('Z'));
        assert_eq!(formatted.len(), 20);
    }

    #[test]
    fn test_format_iso8601_utc_epoch_boundary() {
        let epoch = std::time::UNIX_EPOCH;
        let formatted = format_iso8601_utc(epoch);
        assert_eq!(formatted, "1970-01-01T00:00:00Z");
        assert_eq!(formatted.len(), 20);
    }

    #[test]
    fn test_format_iso8601_utc_into_appends() {
        let mut buf = String::from("PREFIX_");
        format_iso8601_utc_into(std::time::UNIX_EPOCH, &mut buf);
        assert_eq!(buf, "PREFIX_1970-01-01T00:00:00Z");
    }

    #[test]
    fn test_format_iso8601_utc_known_timestamps() {
        use std::time::{Duration, UNIX_EPOCH};

        let t1 = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
        let formatted1 = format_iso8601_utc(t1);
        assert_eq!(formatted1, "2023-11-14T22:13:20Z");
        assert_eq!(formatted1.len(), 20);

        let t2 = UNIX_EPOCH + Duration::from_secs(1_582_934_400);
        let formatted2 = format_iso8601_utc(t2);
        assert_eq!(formatted2, "2020-02-29T00:00:00Z");
        assert_eq!(formatted2.len(), 20);
    }

    #[test]
    fn test_format_iso8601_utc_sub_epoch_fallback() {
        use std::time::{Duration, UNIX_EPOCH};

        if let Some(sub_epoch) = UNIX_EPOCH.checked_sub(Duration::from_secs(3600)) {
            let formatted = format_iso8601_utc(sub_epoch);
            assert_eq!(formatted, "1970-01-01T00:00:00Z");
            assert_eq!(formatted.len(), 20);
        }
    }
}
