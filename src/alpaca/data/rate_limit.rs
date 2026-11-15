use std::time::{Duration, SystemTime, UNIX_EPOCH};

use reqwest::header::HeaderMap;

/// Reads the time until the rate limit resets from Alpaca's `X-RateLimit-Reset` header.
pub(super) fn reset_hint(headers: &HeaderMap, now: SystemTime) -> Option<Duration> {
    headers
        .get("x-ratelimit-reset")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| reset_after(value, now))
}

/// Parses an `X-RateLimit-Reset` value (UNIX epoch seconds) into the time remaining at `now`.
/// Returns `None` when the value is malformed or not in the future.
fn reset_after(header: &str, now: SystemTime) -> Option<Duration> {
    let seconds = header.trim().parse().ok()?;
    UNIX_EPOCH
        .checked_add(Duration::from_secs(seconds))?
        .duration_since(now)
        .ok()
        .filter(|wait| !wait.is_zero())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reset_after_returns_remaining_time_when_header_is_a_future_epoch() {
        let now = UNIX_EPOCH + Duration::from_millis(1_000_500);
        let cases = [
            ("1001", Some(Duration::from_millis(500))),
            ("1060", Some(Duration::from_millis(59_500))),
            (" 1060 ", Some(Duration::from_millis(59_500))),
            ("1000", None),
            ("0", None),
            ("", None),
            ("soon", None),
            ("-5", None),
            ("1060.5", None),
            ("18446744073709551615", None),
            ("18446744073709551616", None),
        ];

        for (header, expected) in cases {
            let wait = reset_after(header, now);

            assert_eq!(wait, expected, "{header:?}");
        }
    }

    #[test]
    fn reset_after_returns_none_when_reset_equals_now() {
        let now = UNIX_EPOCH + Duration::from_secs(1_000);

        let wait = reset_after("1000", now);

        assert_eq!(wait, None);
    }
}
