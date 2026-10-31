//! A budget for caches of data that can be recomputed, kept per key, like the caches of each
//! open document.

use std::hash::Hash;

use parking_lot::Mutex;
use rustc_hash::FxHashMap;

/// Tracks how much each key's caches use, in a unit of the caller's choice, and which keys to
/// evict to stay within `max`: the least recently used first.
///
/// The key that was just used is never evicted, so the caches of a single key can exceed the
/// budget while it is in use. The caller owns the caches: it reports their usage with
/// [`CacheBudget::record`] after using them, and drops the caches of the keys it returns.
pub(crate) struct CacheBudget<K> {
    max: usize,
    state: Mutex<State<K>>,
}

struct State<K> {
    entries: FxHashMap<K, Entry>,
    total: usize,
    /// Incremented on every use, to order the entries by their last use.
    clock: u64,
}

struct Entry {
    usage: usize,
    last_used: u64,
}

impl<K: Eq + Hash + Clone> CacheBudget<K> {
    pub(crate) fn new(max: usize) -> Self {
        Self {
            max,
            state: Mutex::new(State {
                entries: FxHashMap::default(),
                total: 0,
                clock: 0,
            }),
        }
    }

    pub(crate) fn max(&self) -> usize {
        self.max
    }

    /// Records that the caches of `key` were just used and now use `usage`. Returns the keys
    /// whose caches the caller has to drop to get back within the budget, least recently used
    /// first. They are no longer tracked until they are recorded again.
    pub(crate) fn record(&self, key: &K, usage: usize) -> Vec<K> {
        let mut state = self.state.lock();
        state.clock += 1;
        let clock = state.clock;
        let previous = state
            .entries
            .insert(
                key.clone(),
                Entry {
                    usage,
                    last_used: clock,
                },
            )
            .map_or(0, |entry| entry.usage);
        state.total = state.total - previous + usage;
        if state.total <= self.max {
            return Vec::new();
        }

        let mut candidates: Vec<(u64, K)> = state
            .entries
            .iter()
            .filter(|(candidate, _)| *candidate != key)
            .map(|(candidate, entry)| (entry.last_used, candidate.clone()))
            .collect();
        candidates.sort_unstable_by_key(|(last_used, _)| *last_used);

        let mut evicted = Vec::new();
        for (_, candidate) in candidates {
            if state.total <= self.max {
                break;
            }
            if let Some(entry) = state.entries.remove(&candidate) {
                state.total -= entry.usage;
                evicted.push(candidate);
            }
        }
        evicted
    }

    /// Stops tracking `key`, e.g. when its caches are dropped with the document.
    pub(crate) fn remove(&self, key: &K) {
        let mut state = self.state.lock();
        if let Some(entry) = state.entries.remove(key) {
            state.total -= entry.usage;
        }
    }

    #[cfg(test)]
    pub(crate) fn total(&self) -> usize {
        self.state.lock().total
    }
}

#[cfg(test)]
mod tests {
    use super::CacheBudget;

    #[test]
    fn evicts_the_least_recently_used_keys_first() {
        let budget = CacheBudget::new(10);
        assert!(budget.record(&"a", 4).is_empty());
        assert!(budget.record(&"b", 4).is_empty());
        // `a` was used last, so `b` goes first.
        assert!(budget.record(&"a", 4).is_empty());
        assert_eq!(budget.record(&"c", 4), ["b"]);
        assert_eq!(budget.total(), 8);

        // The key in use is never evicted, even alone over the budget.
        assert_eq!(budget.record(&"c", 20), ["a"]);
        assert_eq!(budget.total(), 20);
        assert!(budget.record(&"c", 20).is_empty());

        budget.remove(&"c");
        assert_eq!(budget.total(), 0);
        assert!(budget.record(&"a", 10).is_empty());
    }
}
