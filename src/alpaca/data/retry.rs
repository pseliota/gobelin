use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Retry behaviour for transport errors, HTTP 429 and HTTP 5xx.
///
/// The delay doubles from `base_delay` after each attempt and is capped at `max_delay`.
/// A 429 that carries a rate-limit reset time waits until that time instead,
/// and is not retried when the wait would exceed `max_delay`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryPolicy {
    /// Total attempts including the first. Zero behaves as one.
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
        }
    }
}

impl RetryPolicy {
    /// A policy that never retries.
    pub fn none() -> Self {
        Self {
            max_attempts: 1,
            ..Self::default()
        }
    }

    /// Returns how long to wait before the next attempt, or `None` to give up.
    /// `attempts` counts the attempts already made, including the failed one.
    pub(super) fn delay(&self, attempts: u32, failure: Failure) -> Option<Duration> {
        if attempts >= self.max_attempts.max(1) {
            return None;
        }
        match failure {
            Failure::Status {
                status: 429,
                reset_after: Some(wait),
            } => (wait <= self.max_delay).then_some(wait),
            Failure::Transport
            | Failure::Status {
                status: 429 | 500..=599,
                ..
            } => Some(self.backoff(attempts)),
            Failure::Status { .. } => None,
        }
    }

    fn backoff(&self, attempts: u32) -> Duration {
        2u32.checked_pow(attempts.saturating_sub(1))
            .and_then(|factor| self.base_delay.checked_mul(factor))
            .map_or(self.max_delay, |delay| delay.min(self.max_delay))
    }
}

/// A failed attempt as seen by the retry decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Failure {
    Transport,
    Status {
        status: u16,
        /// Time until the rate limit resets, when the response said so.
        reset_after: Option<Duration>,
    },
}

/// Parses an `X-RateLimit-Reset` value (UNIX epoch seconds) into the time remaining at `now`.
/// Returns `None` when the value is malformed or not in the future.
pub(super) fn reset_after(header: &str, now: SystemTime) -> Option<Duration> {
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

    const fn secs(seconds: u64) -> Duration {
        Duration::from_secs(seconds)
    }

    const fn status(status: u16) -> Failure {
        Failure::Status {
            status,
            reset_after: None,
        }
    }

    const fn rate_limited(reset_after: u64) -> Failure {
        Failure::Status {
            status: 429,
            reset_after: Some(secs(reset_after)),
        }
    }

    fn policy(max_attempts: u32) -> RetryPolicy {
        RetryPolicy {
            max_attempts,
            base_delay: secs(1),
            max_delay: secs(30),
        }
    }

    #[test]
    fn default_allows_three_attempts_when_unconfigured() {
        let policy = RetryPolicy::default();

        assert_eq!(policy.max_attempts, 3);
        assert_eq!(policy.base_delay, secs(1));
        assert_eq!(policy.max_delay, secs(30));
    }

    #[test]
    fn none_allows_one_attempt_when_constructed() {
        let policy = RetryPolicy::none();

        assert_eq!(policy.max_attempts, 1);
    }

    #[test]
    fn delay_retries_only_retryable_failures_when_attempts_remain() {
        let cases = [
            (Failure::Transport, Some(secs(1))),
            (status(429), Some(secs(1))),
            (status(500), Some(secs(1))),
            (status(503), Some(secs(1))),
            (status(599), Some(secs(1))),
            (status(302), None),
            (status(400), None),
            (status(401), None),
            (status(403), None),
            (status(404), None),
            (status(422), None),
            (status(499), None),
            (status(600), None),
        ];
        let policy = policy(3);

        for (failure, expected) in cases {
            let delay = policy.delay(1, failure);

            assert_eq!(delay, expected, "{failure:?}");
        }
    }

    #[test]
    fn delay_doubles_up_to_cap_when_attempts_increase() {
        let cases = [
            (1, secs(1)),
            (2, secs(2)),
            (3, secs(4)),
            (5, secs(16)),
            (6, secs(30)),
            (7, secs(30)),
            (40, secs(30)),
            (u32::MAX - 1, secs(30)),
        ];
        let policy = policy(u32::MAX);

        for (attempts, expected) in cases {
            let delay = policy.delay(attempts, Failure::Transport);

            assert_eq!(delay, Some(expected), "attempts {attempts}");
        }
    }

    #[test]
    fn delay_gives_up_when_max_attempts_were_made() {
        let cases = [
            (0, 1, None),
            (1, 1, None),
            (2, 1, Some(secs(1))),
            (2, 2, None),
            (3, 2, Some(secs(2))),
            (3, 3, None),
            (3, 4, None),
        ];

        for (max_attempts, attempts, expected) in cases {
            let delay = policy(max_attempts).delay(attempts, status(503));

            assert_eq!(delay, expected, "{attempts} of {max_attempts}");
        }
    }

    #[test]
    fn delay_waits_for_rate_limit_reset_when_reset_time_is_known() {
        let cases = [
            (rate_limited(7), 1, Some(secs(7))),
            (rate_limited(30), 1, Some(secs(30))),
            (rate_limited(31), 1, None),
            (rate_limited(7), 2, Some(secs(7))),
            (rate_limited(7), 3, None),
            (
                Failure::Status {
                    status: 503,
                    reset_after: Some(secs(7)),
                },
                1,
                Some(secs(1)),
            ),
            (
                Failure::Status {
                    status: 400,
                    reset_after: Some(secs(7)),
                },
                1,
                None,
            ),
        ];
        let policy = policy(3);

        for (failure, attempts, expected) in cases {
            let delay = policy.delay(attempts, failure);

            assert_eq!(delay, expected, "{failure:?} after {attempts}");
        }
    }

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
        let now = UNIX_EPOCH + secs(1_000);

        let wait = reset_after("1000", now);

        assert_eq!(wait, None);
    }
}
