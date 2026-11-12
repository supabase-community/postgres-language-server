use std::sync::LazyLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// The instant [LastUsed] counts from.
static EPOCH: LazyLock<Instant> = LazyLock::new(Instant::now);

/// When a cached value was last used. It can be updated through a shared reference, so that a
/// cache hit under a read lock still counts as a use.
pub(crate) struct LastUsed(AtomicU64);

impl Default for LastUsed {
    fn default() -> Self {
        Self::now()
    }
}

impl LastUsed {
    pub(crate) fn now() -> Self {
        Self(AtomicU64::new(Self::millis_since_epoch()))
    }

    pub(crate) fn touch(&self) {
        self.0.store(Self::millis_since_epoch(), Ordering::Relaxed);
    }

    pub(crate) fn idle_for(&self) -> Duration {
        Duration::from_millis(
            Self::millis_since_epoch().saturating_sub(self.0.load(Ordering::Relaxed)),
        )
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

    fn millis_since_epoch() -> u64 {
        // The epoch is set on first use, so pretend it was earlier than any recorded use.
        u64::try_from(EPOCH.elapsed().as_millis()).unwrap_or(u64::MAX) + Self::OFFSET_MILLIS
    }

    /// Lets [LastUsed::backdate] go back before the epoch was set.
    const OFFSET_MILLIS: u64 = 24 * 60 * 60 * 1000;
}
