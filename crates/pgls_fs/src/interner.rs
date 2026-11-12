use crossbeam::channel::{Receiver, Sender, unbounded};
use parking_lot::RwLock;
use rustc_hash::FxHashSet;
use std::panic::AssertUnwindSafe;
use std::path::PathBuf;

/// File paths interner cache
///
/// The path interner stores an instance of [PathBuf]
pub struct PathInterner {
    /// A panic while inserting leaves the set valid, so it can cross an unwind boundary like the
    /// files of [crate::MemoryFileSystem].
    storage: AssertUnwindSafe<RwLock<FxHashSet<PathBuf>>>,
    handler: Sender<PathBuf>,
}

impl PathInterner {
    pub fn new() -> (Self, Receiver<PathBuf>) {
        let (send, recv) = unbounded();
        let interner = Self {
            storage: AssertUnwindSafe(RwLock::new(FxHashSet::default())),
            handler: send,
        };

        (interner, recv)
    }

    /// Insert the path.
    /// Returns `true` if the path was not previously inserted.
    pub fn intern_path(&self, path: PathBuf) -> bool {
        let result = self.storage.write().insert(path.clone());
        if result {
            self.handler.send(path).ok();
        }
        result
    }
}
