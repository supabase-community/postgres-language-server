use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use super::runtime::Clock;

/// When a cached value was last used, on the workspace's [Clock]. It can be updated through a
/// shared reference, so that a cache hit under a read lock still counts as a use.
pub(crate) struct LastUsed(AtomicU64);

impl LastUsed {
    pub(crate) fn now(clock: &Clock) -> Self {
        Self(AtomicU64::new(Self::millis(clock)))
    }

    pub(crate) fn touch(&self, clock: &Clock) {
        self.0.store(Self::millis(clock), Ordering::Relaxed);
    }

    pub(crate) fn idle_for(&self, clock: &Clock) -> Duration {
        Duration::from_millis(Self::millis(clock).saturating_sub(self.0.load(Ordering::Relaxed)))
    }

    /// Pretends it was last used `duration` earlier, for tests of idle eviction.
    #[cfg(test)]
    pub(crate) fn backdate(&self, duration: Duration) {
        let millis = u64::try_from(duration.as_millis()).unwrap_or(u64::MAX);
        self.0.fetch_sub(
            millis.min(self.0.load(Ordering::Relaxed)),
            Ordering::Relaxed,
        );
    }

    /// The milliseconds since the clock was created. Pretends it was created earlier than any
    /// recorded use.
    fn millis(clock: &Clock) -> u64 {
        u64::try_from(clock.elapsed().as_millis()).unwrap_or(u64::MAX) + Self::OFFSET_MILLIS
    }

    /// Lets [LastUsed::backdate] go back before the clock was created.
    const OFFSET_MILLIS: u64 = 24 * 60 * 60 * 1000;
}
