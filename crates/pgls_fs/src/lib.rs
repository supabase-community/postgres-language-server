//! # pgls_fs

mod dir;
mod fs;
mod interner;
mod path;

pub use dir::ensure_cache_dir;
pub use interner::PathInterner;
pub use path::PgLSPath;

pub use fs::{
    AutoSearchResult, ConfigName, ErrorEntry, File, FileSystem, FileSystemDiagnostic,
    FileSystemExt, MemoryFileSystem, OpenOptions, OsFileSystem, TraversalContext, TraversalScope,
};
