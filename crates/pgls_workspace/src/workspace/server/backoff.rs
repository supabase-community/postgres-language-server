use std::time::{Duration, Instant};

/// How long the workspace waits before it retries a database operation that failed: `initial`
/// after the first failure, doubling with every failure in a row, up to `max`.
///
/// The defaults are 5 seconds and a minute. Tests set them short, so that they can wait for a
/// retry on the real clock.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BackoffDurations {
    pub initial: Duration,
    pub max: Duration,
}

impl Default for BackoffDurations {
    fn default() -> Self {
        Self {
            initial: Duration::from_secs(5),
            max: Duration::from_secs(60),
        }
    }
}

/// How long to wait before retrying an operation that failed, see [BackoffDurations].
pub(crate) struct Backoff {
    attempts: u32,
    delay: Duration,
    next_retry_at: Instant,
}

impl Backoff {
    /// The backoff after another failure, following the failures in a row before it, if any.
    pub(crate) fn after_failure(previous: Option<&Backoff>, durations: BackoffDurations) -> Self {
        let attempts = previous.map_or(1, |backoff| backoff.attempts.saturating_add(1));
        let multiplier = 1u32
            .checked_shl(attempts.saturating_sub(1))
            .unwrap_or(u32::MAX);
        let delay = durations
            .initial
            .saturating_mul(multiplier)
            .min(durations.max);
        Self {
            attempts,
            delay,
            next_retry_at: Instant::now() + delay,
        }
    }

    /// Whether the operation should not be retried yet.
    pub(crate) fn is_active(&self) -> bool {
        Instant::now() < self.next_retry_at
    }

    /// When the operation may be retried.
    pub(crate) fn next_retry_at(&self) -> Instant {
        self.next_retry_at
    }

    /// How long the operation isn't retried after the failure.
    pub(crate) fn delay(&self) -> Duration {
        self.delay
    }

    /// Lets the operation be retried now, for tests.
    #[cfg(test)]
    pub(crate) fn expire(&mut self) {
        self.next_retry_at = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doubles_up_to_a_minute() {
        let durations = BackoffDurations::default();
        let mut backoff = Backoff::after_failure(None, durations);
        let mut delays = vec![];
        for _ in 0..6 {
            delays.push(backoff.delay().as_secs());
            backoff = Backoff::after_failure(Some(&backoff), durations);
        }
        assert_eq!(delays, [5, 10, 20, 40, 60, 60]);
        assert!(backoff.is_active());

        backoff.expire();
        assert!(!backoff.is_active());
    }

    /// With short durations, a test waits for the retry on the real clock.
    #[test]
    fn short_durations_expire_on_the_real_clock() {
        let durations = BackoffDurations {
            initial: Duration::from_millis(10),
            max: Duration::from_millis(15),
        };
        let first = Backoff::after_failure(None, durations);
        let second = Backoff::after_failure(Some(&first), durations);
        assert_eq!(second.delay(), Duration::from_millis(15));
        assert!(second.is_active());

        std::thread::sleep(Duration::from_millis(20));
        assert!(!second.is_active());
    }
}
