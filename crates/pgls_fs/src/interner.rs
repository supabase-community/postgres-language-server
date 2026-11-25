use parking_lot::RwLock;
use rustc_hash::FxHashSet;
use std::panic::AssertUnwindSafe;
use std::path::PathBuf;

/// The paths a traversal has seen, so that it handles each path once, also when it reaches it
/// through a symbolic link.
#[derive(Default)]
pub struct PathInterner {
    /// A panic while inserting leaves the set valid, so it can cross an unwind boundary like the
    /// files of [crate::MemoryFileSystem].
    storage: AssertUnwindSafe<RwLock<FxHashSet<PathBuf>>>,
}

impl PathInterner {
    /// Insert the path.
    /// Returns `true` if the path was not previously inserted.
    pub fn intern_path(&self, path: PathBuf) -> bool {
        self.storage.write().insert(path)
    }
}
