use std::time::{Duration, Instant};

const INITIAL_BACKOFF: Duration = Duration::from_secs(5);
const MAX_BACKOFF: Duration = Duration::from_secs(60);

/// How long to wait before retrying an operation that failed: 5 seconds after the first failure,
/// doubling with every failure in a row, up to a minute.
pub(crate) struct Backoff {
    attempts: u32,
    delay: Duration,
    next_retry_at: Instant,
}

impl Backoff {
    /// The backoff after another failure, following the failures in a row before it, if any.
    pub(crate) fn after_failure(previous: Option<&Backoff>) -> Self {
        let attempts = previous.map_or(1, |backoff| backoff.attempts.saturating_add(1));
        let multiplier = 1u32
            .checked_shl(attempts.saturating_sub(1))
            .unwrap_or(u32::MAX);
        let delay = INITIAL_BACKOFF.saturating_mul(multiplier).min(MAX_BACKOFF);
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
        let mut backoff = Backoff::after_failure(None);
        let mut delays = vec![];
        for _ in 0..6 {
            delays.push(backoff.delay().as_secs());
            backoff = Backoff::after_failure(Some(&backoff));
        }
        assert_eq!(delays, [5, 10, 20, 40, 60, 60]);
        assert!(backoff.is_active());

        backoff.expire();
        assert!(!backoff.is_active());
    }
}
