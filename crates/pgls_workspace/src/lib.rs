use std::ops::{Deref, DerefMut};

use pgls_console::Console;
use pgls_fs::{FileSystem, OsFileSystem};

pub mod configuration;
pub mod diagnostics;
pub mod dome;
pub mod features;
pub mod matcher;
pub mod settings;
pub mod workspace;

#[cfg(feature = "schema")]
pub mod workspace_types;

pub use crate::configuration::PartialConfigurationExt;
pub use crate::diagnostics::{TransportError, WorkspaceError};
pub use crate::workspace::Workspace;

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
