//! The overlay's maps, with an undo log while a transaction is open.
//!
//! `BEGIN` and `SAVEPOINT` remember the length of each map's log, in O(1), and `ROLLBACK [TO]`
//! replays the log backwards to it, in O(changes since the savepoint). Neither depends on the
//! size of the file, so a file with a transaction every few statements stays linear.

use std::borrow::Borrow;
use std::hash::Hash;
use std::ops::Deref;

use rustc_hash::FxHashMap;

/// A map of the overlay. Reads go through [`Deref`]; every write goes through a method that
/// records the previous value of the key while the map is recording.
#[derive(Debug, Clone)]
pub(super) struct JournaledMap<K, V> {
    map: FxHashMap<K, V>,
    /// Each write since the outermost savepoint: the key and its value before the write, `None`
    /// if it had none. Empty unless [`JournaledMap::recording`].
    undo: Vec<(K, Option<V>)>,
    recording: bool,
}

impl<K, V> Default for JournaledMap<K, V> {
    fn default() -> Self {
        Self {
            map: FxHashMap::default(),
            undo: Vec::new(),
            recording: false,
        }
    }
}

impl<K, V> Deref for JournaledMap<K, V> {
    type Target = FxHashMap<K, V>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl<'a, K, V> IntoIterator for &'a JournaledMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = std::collections::hash_map::Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.iter()
    }
}

impl<K: Eq + Hash + Clone, V: Clone> JournaledMap<K, V> {
    pub(super) fn insert(&mut self, key: K, value: V) -> Option<V> {
        if !self.recording {
            return self.map.insert(key, value);
        }
        let previous = self.map.insert(key.clone(), value);
        self.undo.push((key, previous.clone()));
        previous
    }

    pub(super) fn remove(&mut self, key: &K) -> Option<V> {
        let (key, value) = self.map.remove_entry(key)?;
        if self.recording {
            self.undo.push((key, Some(value.clone())));
        }
        Some(value)
    }

    pub(super) fn retain(&mut self, mut keep: impl FnMut(&K, &V) -> bool) {
        let Self {
            map,
            undo,
            recording,
        } = self;
        map.retain(|key, value| {
            let keep = keep(key, value);
            if !keep && *recording {
                undo.push((key.clone(), Some(value.clone())));
            }
            keep
        });
    }

    pub(super) fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        if self.recording
            && let Some((key, value)) = self.map.get_key_value(key)
        {
            self.undo.push((key.clone(), Some(value.clone())));
        }
        self.map.get_mut(key)
    }

    /// The value of `key`, inserting the default value if it has none.
    pub(super) fn get_or_default(&mut self, key: K) -> &mut V
    where
        V: Default,
    {
        if self.recording {
            self.undo.push((key.clone(), self.map.get(&key).cloned()));
        }
        self.map.entry(key).or_default()
    }

    /// Calls `update` on every value that `matches`.
    pub(super) fn update_matching(
        &mut self,
        matches: impl Fn(&V) -> bool,
        mut update: impl FnMut(&mut V),
    ) {
        for (key, value) in &mut self.map {
            if !matches(value) {
                continue;
            }
            if self.recording {
                self.undo.push((key.clone(), Some(value.clone())));
            }
            update(value);
        }
    }
}

/// A set of the overlay, recorded like [`JournaledMap`].
#[derive(Debug, Clone, Default)]
pub(super) struct JournaledSet<K> {
    map: JournaledMap<K, ()>,
}

impl<K: Eq + Hash + Clone> JournaledSet<K> {
    pub(super) fn contains<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.map.contains_key(key)
    }

    /// Whether the set didn't contain `key`.
    pub(super) fn insert(&mut self, key: K) -> bool {
        self.map.insert(key, ()).is_none()
    }

    /// Whether the set contained `key`.
    pub(super) fn remove(&mut self, key: &K) -> bool {
        self.map.remove(key).is_some()
    }
}

impl<'a, K> IntoIterator for &'a JournaledSet<K> {
    type Item = &'a K;
    type IntoIter = std::collections::hash_map::Keys<'a, K, ()>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.map.keys()
    }
}

/// The undo log of a [`JournaledMap`] or [`JournaledSet`], as the transaction statements use it.
pub(super) trait Journal {
    /// Starts recording writes, or stops and forgets the log.
    fn set_recording(&mut self, recording: bool);
    /// How many writes the log holds.
    fn undo_len(&self) -> usize;
    /// Undoes the writes after the first `len`.
    fn undo_to(&mut self, len: usize);
}

impl<K: Eq + Hash, V> Journal for JournaledMap<K, V> {
    fn set_recording(&mut self, recording: bool) {
        self.recording = recording;
        if !recording {
            self.undo = Vec::new();
        }
    }

    fn undo_len(&self) -> usize {
        self.undo.len()
    }

    fn undo_to(&mut self, len: usize) {
        debug_assert!(len <= self.undo.len(), "undo past the end of the log");
        for (key, previous) in self.undo.drain(len.min(self.undo.len())..).rev() {
            match previous {
                Some(value) => self.map.insert(key, value),
                None => self.map.remove(&key),
            };
        }
    }
}

impl<K: Eq + Hash> Journal for JournaledSet<K> {
    fn set_recording(&mut self, recording: bool) {
        self.map.set_recording(recording);
    }

    fn undo_len(&self) -> usize {
        self.map.undo_len()
    }

    fn undo_to(&mut self, len: usize) {
        self.map.undo_to(len);
    }
}
