//! # pgls_fs

// The file system is external data, so a panic on an unexpected file takes down the server for
// every connected editor.
#![warn(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

mod dir;
mod fs;
mod interner;
mod path;

pub use dir::ensure_cache_dir;
pub use interner::PathInterner;
pub use path::PgLSPath;

pub use fs::{
    AutoSearchResult, ConfigName, ErrorEntry, File, FileSystem, FileSystemDiagnostic,
    FileSystemExt, FindFiles, FoundFiles, MemoryFileSystem, OpenOptions, OsFileSystem,
    SKIPPED_DIRECTORIES, SkipDirectory, TraversalContext, TraversalScope,
};
