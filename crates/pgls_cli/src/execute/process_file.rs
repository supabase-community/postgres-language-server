mod check;
pub(crate) mod workspace_file;

use crate::execute::config::ExecutionMode;
use crate::execute::walk::TraversalOptions;
use check::check_file;
use pgls_diagnostics::Error;
use pgls_fs::PgLSPath;
use std::marker::PhantomData;
use std::ops::Deref;

#[derive(Debug)]
pub(crate) enum FileStatus {
    /// File changed and it was a success
    Changed,
    /// File unchanged, and it was a success
    Unchanged,

    /// While handling the file, something happened
    #[allow(unused)]
    Message(Message),

    /// A match was found while searching a file
    #[allow(unused)]
    SearchResult(usize, Message),

    /// File ignored, it should not be count as "handled"
    #[allow(unused)]
    Ignored,

    /// Files that belong to other tools and shouldn't be touched
    #[allow(unused)]
    Protected(String),
}

/// Wrapper type for messages that can be printed during the traversal process
#[derive(Debug)]
pub(crate) enum Message {
    #[allow(unused)]
    SkippedFixes {
        /// Suggested fixes skipped during the lint traversal
        skipped_suggested_fixes: u32,
    },

    #[allow(unused)]
    Failure,
    Error(Error),
    Diagnostics {
        name: String,
        content: String,
        diagnostics: Vec<Error>,
        skipped_diagnostics: u32,
    },
}

impl<D> From<D> for Message
where
    Error: From<D>,
    D: std::fmt::Debug,
{
    fn from(err: D) -> Self {
        Self::Error(Error::from(err))
    }
}

/// The return type for [process_file], with the following semantics:
/// - `Ok(Success)` means the operation was successful (the file is added to
///   the `processed` counter)
/// - `Ok(Message(_))` means the operation was successful but a message still
///   needs to be printed (eg. the diff when not in CI or write mode)
/// - `Ok(Ignored)` means the file was ignored (the file is not added to the
///   `processed` or `skipped` counters)
/// - `Err(_)` means the operation failed and the file should be added to the
///   `skipped` counter
pub(crate) type FileResult = Result<FileStatus, Message>;

/// Data structure that allows to pass [TraversalOptions] to multiple consumers, bypassing the
/// compiler constraints set by the lifetimes of the [TraversalOptions]
pub(crate) struct SharedTraversalOptions<'ctx, 'app> {
    inner: &'app TraversalOptions<'ctx, 'app>,
    _p: PhantomData<&'app ()>,
}

impl<'ctx, 'app> SharedTraversalOptions<'ctx, 'app> {
    fn new(t: &'app TraversalOptions<'ctx, 'app>) -> Self {
        Self {
            _p: PhantomData,
            inner: t,
        }
    }
}

impl<'ctx, 'app> Deref for SharedTraversalOptions<'ctx, 'app> {
    type Target = TraversalOptions<'ctx, 'app>;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

/// This function performs the actual processing: it reads the file from disk
/// and parse it; analyze and / or format it; then it either fails if error
/// diagnostics were emitted, or compare the formatted code with the original
/// content of the file and emit a diff or write the new content to the disk if
/// write mode is enabled
pub(crate) fn process_file(ctx: &TraversalOptions, pgls_path: &PgLSPath) -> FileResult {
    tracing::trace_span!("process_file", path = ?pgls_path).in_scope(move || {
        let shared_context = &SharedTraversalOptions::new(ctx);

        match ctx.config.mode {
            ExecutionMode::Check { .. } => check_file(shared_context, pgls_path),
        }
    })
}
