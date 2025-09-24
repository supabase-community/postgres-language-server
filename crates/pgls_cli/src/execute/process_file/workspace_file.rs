use crate::execute::diagnostics::{ResultExt, ResultIoExt};
use crate::execute::process_file::SharedTraversalOptions;
use pgls_diagnostics::{Error, category};
use pgls_fs::{File, OpenOptions, PgLSPath};
use pgls_workspace::workspace::{FileGuard, OpenFileParams};
use pgls_workspace::{Workspace, WorkspaceError};
use std::path::{Path, PathBuf};

/// Small wrapper that holds information and operations around the current processed file
pub(crate) struct WorkspaceFile<'ctx, 'app> {
    guard: FileGuard<'app, dyn Workspace + 'ctx>,
    #[allow(dead_code)]
    file: Box<dyn File>,
    pub(crate) path: PathBuf,
}

impl<'ctx, 'app> WorkspaceFile<'ctx, 'app> {
    /// It attempts to read the file from disk, creating a [FileGuard] and
    /// saving these information internally
    pub(crate) fn new(
        ctx: &SharedTraversalOptions<'ctx, 'app>,
        path: &Path,
    ) -> Result<Self, Error> {
        let pgt_path = PgLSPath::new(path);
        let open_options = OpenOptions::default()
            .read(true)
            .write(ctx.execution.requires_write_access());
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
                path: pgt_path,
                version: 0,
                content: input.clone(),
            },
        )
        .with_file_path_and_code(path.display().to_string(), category!("internalError/fs"))?;

        Ok(Self {
            file,
            guard,
            path: PathBuf::from(path),
        })
    }

    pub(crate) fn guard(&self) -> &FileGuard<'app, dyn Workspace + 'ctx> {
        &self.guard
    }

    pub(crate) fn input(&self) -> Result<String, WorkspaceError> {
        self.guard().get_file_content()
    }
}
