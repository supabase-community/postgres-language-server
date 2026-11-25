use crate::execute::diagnostics::{ResultExt, ResultIoExt};
use crate::execute::walk::TraversalOptions;
use pgls_diagnostics::{Error, category};
use pgls_fs::{File, OpenOptions, PgLSPath};
use pgls_workspace::workspace::{FileGuard, OpenFileParams};
use pgls_workspace::{Workspace, WorkspaceError};
use std::path::{Path, PathBuf};

/// Small wrapper that holds information and operations around the current processed file
pub(crate) struct WorkspaceFile<'ctx> {
    guard: FileGuard<'ctx, dyn Workspace + 'ctx>,
    /// The file on disk, opened through the file system of the traversal, so that the writes go
    /// through it too.
    file: Box<dyn File>,
    pub(crate) path: PathBuf,
}

impl<'ctx> WorkspaceFile<'ctx> {
    /// It attempts to read the file from disk, creating a [FileGuard] and
    /// saving these information internally
    pub(crate) fn new(ctx: &TraversalOptions<'ctx, '_>, path: &Path) -> Result<Self, Error> {
        let pgls_path = PgLSPath::new(path);
        let open_options = OpenOptions::default()
            .read(true)
            .write(ctx.config.allows_writes());
        let mut file = ctx
            .fs
            .open_with_options(path, open_options)
            .with_file_path(path.display().to_string())?;

        let mut input = String::new();
        file.read_to_string(&mut input)
            .with_file_path(path.display().to_string())?;

        let guard = FileGuard::open(
            ctx.workspace,
            OpenFileParams {
                project_key: ctx.project_key,
                path: pgls_path,
                version: 0,
                content: input,
            },
        )
        .with_file_path_and_code(path.display().to_string(), category!("internalError/fs"))?;

        Ok(Self {
            file,
            guard,
            path: PathBuf::from(path),
        })
    }

    pub(crate) fn guard(&self) -> &FileGuard<'ctx, dyn Workspace + 'ctx> {
        &self.guard
    }

    pub(crate) fn input(&self) -> Result<String, WorkspaceError> {
        self.guard().get_file_content()
    }

    /// Writes `content` to the file on disk.
    pub(crate) fn write(&mut self, content: &str) -> Result<(), Error> {
        self.file
            .set_content(content.as_bytes())
            .with_file_path(self.path.display().to_string())
    }
}
