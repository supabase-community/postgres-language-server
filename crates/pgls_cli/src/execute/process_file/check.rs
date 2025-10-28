use pgls_analyse::RuleCategoriesBuilder;
use pgls_diagnostics::{Error, category};

use crate::execute::diagnostics::ResultExt;
use crate::execute::process_file::workspace_file::WorkspaceFile;
use crate::execute::process_file::{FileResult, FileStatus, Message, SharedTraversalOptions};
use std::path::Path;
use std::sync::atomic::Ordering;

/// Lints a single file and returns a [FileResult]
pub(crate) fn check_file<'ctx>(
    ctx: &'ctx SharedTraversalOptions<'ctx, '_>,
    path: &Path,
) -> FileResult {
    let mut workspace_file = WorkspaceFile::new(ctx, path)?;
    check_with_guard(ctx, &mut workspace_file)
}

pub(crate) fn check_with_guard<'ctx>(
    ctx: &'ctx SharedTraversalOptions<'ctx, '_>,
    workspace_file: &mut WorkspaceFile,
) -> FileResult {
    tracing::info_span!("Processes check", path =? workspace_file.path.display()).in_scope(
        move || {
            let input = workspace_file.input()?;
            let changed = false;

            let (only, skip) = (Vec::new(), Vec::new());

            let max_diagnostics = ctx.remaining_diagnostics.load(Ordering::Relaxed);

            let pull_diagnostics_result = workspace_file
                .guard()
                .pull_diagnostics(
                    RuleCategoriesBuilder::default().all().build(),
                    max_diagnostics,
                    only,
                    skip,
                )
                .with_file_path_and_code(
                    workspace_file.path.display().to_string(),
                    category!("check"),
                )?;

            let no_diagnostics = pull_diagnostics_result.diagnostics.is_empty()
                && pull_diagnostics_result.skipped_diagnostics == 0;

            if !no_diagnostics {
                ctx.push_message(Message::Diagnostics {
                    name: workspace_file.path.display().to_string(),
                    content: input,
                    diagnostics: pull_diagnostics_result
                        .diagnostics
                        .into_iter()
                        .map(Error::from)
                        .collect(),
                    skipped_diagnostics: pull_diagnostics_result.skipped_diagnostics as u32,
                });
            }

            if changed {
                Ok(FileStatus::Changed)
            } else {
                Ok(FileStatus::Unchanged)
            }
        },
    )
}
