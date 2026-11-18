//! # pgls_fs

mod dir;
mod fs;
mod interner;
mod path;

pub use dir::ensure_cache_dir;
pub use interner::PathInterner;
pub use path::{PgLSPath, normalize_path};

pub use fs::{
    AutoSearchResult, ConfigName, ErrorEntry, File, FileSystem, FileSystemDiagnostic,
    FileSystemExt, FindFiles, FoundFiles, MemoryFileSystem, NotAGitRepository, OpenOptions,
    OsFileSystem, SKIPPED_DIRECTORIES, SkipDirectory, TraversalContext, TraversalScope,
};
