use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// When a cached value was last used. It can be updated through a shared reference, so that a
/// cache hit under a read lock still counts as a use.
pub(crate) struct LastUsed {
    /// The instant the uses are counted from: when this was created.
    epoch: Instant,
    /// The milliseconds from `epoch` to the last use, plus [LastUsed::OFFSET_MILLIS].
    millis: AtomicU64,
}

impl Default for LastUsed {
    fn default() -> Self {
        Self::now()
    }
}

impl LastUsed {
    pub(crate) fn now() -> Self {
        Self {
            epoch: Instant::now(),
            millis: AtomicU64::new(Self::OFFSET_MILLIS),
        }
    }

    pub(crate) fn touch(&self) {
        self.millis.store(self.millis_now(), Ordering::Relaxed);
    }

    pub(crate) fn idle_for(&self) -> Duration {
        Duration::from_millis(
            self.millis_now()
                .saturating_sub(self.millis.load(Ordering::Relaxed)),
        )
    }

    /// Pretends it was last used `duration` earlier, for tests of idle eviction.
    #[cfg(test)]
    pub(crate) fn backdate(&self, duration: Duration) {
        let millis = u64::try_from(duration.as_millis()).unwrap_or(u64::MAX);
        self.millis.fetch_sub(
            millis.min(self.millis.load(Ordering::Relaxed)),
            Ordering::Relaxed,
        );
    }

    /// The milliseconds since the epoch, plus the offset.
    fn millis_now(&self) -> u64 {
        u64::try_from(self.epoch.elapsed().as_millis())
            .unwrap_or(u64::MAX)
            .saturating_add(Self::OFFSET_MILLIS)
    }

    /// Lets [LastUsed::backdate] go back before the epoch.
    const OFFSET_MILLIS: u64 = 24 * 60 * 60 * 1000;
}
