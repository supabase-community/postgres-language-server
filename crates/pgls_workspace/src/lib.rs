use std::ops::{Deref, DerefMut};

use pgls_console::Console;
use pgls_fs::{FileSystem, OsFileSystem};

pub mod configuration;
pub mod diagnostics;
pub mod dome;
pub mod features;
pub mod settings;
pub mod workspace;

#[cfg(feature = "schema")]
pub mod workspace_types;

pub use crate::configuration::PartialConfigurationExt;
pub use crate::diagnostics::{TransportError, WorkspaceError};
pub use crate::workspace::Workspace;

/// The stack size of the threads that run the workspace: the tokio workers and blocking threads
/// of the language server, the rayon workers of `check`, the thread of the other CLI commands,
/// and the runtime of the workspace's database queries.
///
/// The parse tree is decoded and walked recursively (type inference, lint rules, hover, the
/// formatter), so the stack a statement needs grows with its nesting, which `pgls_query` limits
/// to `MAX_NESTING`: as deep as Postgres accepts with its default `max_stack_depth`. A stack
/// overflow aborts the whole process. Running every feature on the deepest statements of several
/// shapes, the most stack measured was 27 MiB in a release build (8,187 `not`s) and 56 MiB in a
/// debug build (8,187 terms of `+`), where the frames are larger.
///
/// The stack is virtual memory that is only committed as it is used, so a thread that never
/// sees such a statement doesn't cost more than with a small stack.
pub const WORKER_STACK_SIZE_BYTES: usize = if cfg!(debug_assertions) {
    256 * 1024 * 1024
} else {
    64 * 1024 * 1024
};

/// This is the main entrypoint of the application.
pub struct App<'app> {
    /// A reference to the internal virtual file system
    pub fs: DynRef<'app, dyn FileSystem>,
    /// A reference to the internal workspace
    pub workspace: WorkspaceRef<'app>,
    /// A reference to the internal console, where its buffer will be used to write messages and
    /// errors
    pub console: &'app mut dyn Console,
}

impl<'app> App<'app> {
    pub fn with_console(console: &'app mut dyn Console) -> Self {
        Self::with_filesystem_and_console(DynRef::Owned(Box::<OsFileSystem>::default()), console)
    }

    /// Create a new instance of the app using the specified [FileSystem] and [Console] implementation
    pub fn with_filesystem_and_console(
        fs: DynRef<'app, dyn FileSystem>,
        console: &'app mut dyn Console,
    ) -> Self {
        Self::new(fs, console, WorkspaceRef::Owned(workspace::server()))
    }

    /// Create a new instance of the app using the specified [FileSystem], [Console] and [Workspace] implementation
    pub fn new(
        fs: DynRef<'app, dyn FileSystem>,
        console: &'app mut dyn Console,
        workspace: WorkspaceRef<'app>,
    ) -> Self {
        Self {
            fs,
            console,
            workspace,
        }
    }
}

pub enum WorkspaceRef<'app> {
    Owned(Box<dyn Workspace>),
    Borrowed(&'app dyn Workspace),
}

impl<'app> Deref for WorkspaceRef<'app> {
    type Target = dyn Workspace + 'app;

    // False positive
    #[allow(clippy::explicit_auto_deref)]
    fn deref(&self) -> &Self::Target {
        match self {
            WorkspaceRef::Owned(inner) => &**inner,
            WorkspaceRef::Borrowed(inner) => *inner,
        }
    }
}

/// Clone of [std::borrow::Cow] specialized for storing a trait object and
/// holding a mutable reference in the `Borrowed` variant instead of requiring
/// the inner type to implement [std::borrow::ToOwned]
pub enum DynRef<'app, T: ?Sized + 'app> {
    Owned(Box<T>),
    Borrowed(&'app mut T),
}

impl<'app, T: ?Sized + 'app> Deref for DynRef<'app, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            DynRef::Owned(inner) => inner,
            DynRef::Borrowed(inner) => inner,
        }
    }
}

impl<'app, T: ?Sized + 'app> DerefMut for DynRef<'app, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            DynRef::Owned(inner) => inner,
            DynRef::Borrowed(inner) => inner,
        }
    }
}
