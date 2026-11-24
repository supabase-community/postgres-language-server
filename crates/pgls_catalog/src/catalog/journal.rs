//! The overlay's maps, with an undo log while a transaction is open.
//!
//! `BEGIN` and `SAVEPOINT` remember the length of each map's log, in O(1), and `ROLLBACK [TO]`
//! replays the log backwards to it, in O(changes since the savepoint). Neither depends on the
//! size of the file, so a file with a transaction every few statements stays linear.

use std::borrow::Borrow;
use std::hash::Hash;
use std::ops::Deref;

use rustc_hash::FxHashMap;

use super::Key;

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

/// The inheritance children and partitions created or attached by the file, by parent. The undo
/// log records each edge that changes, not the parent's whole list, so adding n children to a
/// parent in a transaction stays O(n).
#[derive(Debug, Clone, Default)]
pub(super) struct JournaledChildren {
    map: FxHashMap<Key, Vec<Key>>,
    /// Each edit since the outermost savepoint. Empty unless [`JournaledChildren::recording`].
    undo: Vec<ChildEdit>,
    recording: bool,
}

/// An edit of [`JournaledChildren`], with what it takes to undo it.
#[derive(Debug, Clone)]
enum ChildEdit {
    /// A child was pushed onto the list of `parent`, which `created`.
    Pushed { parent: Key, created: bool },
    /// `child` was removed from position `index` of the list of `parent`.
    Removed {
        parent: Key,
        index: usize,
        child: Key,
    },
    /// The child at position `index` of the list of `parent` was `child` before a rename.
    Renamed {
        parent: Key,
        index: usize,
        child: Key,
    },
    /// The whole list of `parent` was removed.
    RemovedList { parent: Key, children: Vec<Key> },
    /// The list of `from` moved to `to`.
    MovedList { from: Key, to: Key },
}

impl Deref for JournaledChildren {
    type Target = FxHashMap<Key, Vec<Key>>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl JournaledChildren {
    /// Adds `child` to the children of `parent`.
    pub(super) fn push(&mut self, parent: Key, child: Key) {
        let created = !self.map.contains_key(&parent);
        if self.recording {
            self.undo.push(ChildEdit::Pushed {
                parent: parent.clone(),
                created,
            });
        }
        self.map.entry(parent).or_default().push(child);
    }

    /// Removes `child` from the children of `parent`.
    pub(super) fn remove(&mut self, parent: &Key, child: &Key) {
        let Some(children) = self.map.get_mut(parent) else {
            return;
        };
        Self::remove_from(parent, children, child, self.recording, &mut self.undo);
    }

    /// Removes `child` from the children of every parent.
    pub(super) fn remove_everywhere(&mut self, child: &Key) {
        for (parent, children) in &mut self.map {
            Self::remove_from(parent, children, child, self.recording, &mut self.undo);
        }
    }

    /// Removes every `child` from `children`, from the back, so that undoing the removals in
    /// reverse puts each back at its position.
    fn remove_from(
        parent: &Key,
        children: &mut Vec<Key>,
        child: &Key,
        recording: bool,
        undo: &mut Vec<ChildEdit>,
    ) {
        for index in (0..children.len()).rev() {
            if children.get(index) != Some(child) {
                continue;
            }
            let child = children.remove(index);
            if recording {
                undo.push(ChildEdit::Removed {
                    parent: parent.clone(),
                    index,
                    child,
                });
            }
        }
    }

    /// Renames the child `old` to `new` in the children of every parent.
    pub(super) fn rename_everywhere(&mut self, old: &Key, new: &Key) {
        for (parent, children) in &mut self.map {
            for (index, child) in children.iter_mut().enumerate() {
                if child != old {
                    continue;
                }
                let child = std::mem::replace(child, new.clone());
                if self.recording {
                    self.undo.push(ChildEdit::Renamed {
                        parent: parent.clone(),
                        index,
                        child,
                    });
                }
            }
        }
    }

    /// Removes the list of children of `parent`.
    pub(super) fn remove_list(&mut self, parent: &Key) {
        let Some((parent, children)) = self.map.remove_entry(parent) else {
            return;
        };
        if self.recording {
            self.undo.push(ChildEdit::RemovedList { parent, children });
        }
    }

    /// Moves the list of children of `from` to `to`, replacing the list of `to`.
    pub(super) fn move_list(&mut self, from: &Key, to: Key) {
        let Some((from, children)) = self.map.remove_entry(from) else {
            return;
        };
        self.remove_list(&to);
        self.map.insert(to.clone(), children);
        if self.recording {
            self.undo.push(ChildEdit::MovedList { from, to });
        }
    }
}

impl Journal for JournaledChildren {
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
        for edit in self.undo.drain(len.min(self.undo.len())..).rev() {
            match edit {
                ChildEdit::Pushed { parent, created } => {
                    let Some(children) = self.map.get_mut(&parent) else {
                        continue;
                    };
                    children.pop();
                    if created && children.is_empty() {
                        self.map.remove(&parent);
                    }
                }
                ChildEdit::Removed {
                    parent,
                    index,
                    child,
                } => {
                    let children = self.map.entry(parent).or_default();
                    children.insert(index.min(children.len()), child);
                }
                ChildEdit::Renamed {
                    parent,
                    index,
                    child,
                } => {
                    if let Some(slot) = self
                        .map
                        .get_mut(&parent)
                        .and_then(|children| children.get_mut(index))
                    {
                        *slot = child;
                    }
                }
                ChildEdit::RemovedList { parent, children } => {
                    self.map.insert(parent, children);
                }
                ChildEdit::MovedList { from, to } => {
                    if let Some(children) = self.map.remove(&to) {
                        self.map.insert(from, children);
                    }
                }
            }
        }
    }
}
