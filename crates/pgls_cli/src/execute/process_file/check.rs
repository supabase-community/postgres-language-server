use pgls_analyse::RuleCategoriesBuilder;
use pgls_diagnostics::{Error, category};

use crate::execute::config::ExecutionMode;
use crate::execute::diagnostics::ResultExt;
use crate::execute::process_file::workspace_file::WorkspaceFile;
use crate::execute::process_file::{FileResult, FileStatus, Message, SharedTraversalOptions};
use pgls_fs::PgLSPath;
use pgls_workspace::WorkspaceError;
use pgls_workspace::features::code_actions::FixFileParams;
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
            let mut input = workspace_file.input()?;
            let mut changed = false;

            let ExecutionMode::Check {
                fix, only, skip, ..
            } = &ctx.config.mode
            else {
                return Ok(FileStatus::Unchanged);
            };

            if let Some(fix_file_mode) = *fix {
                let fixed = ctx
                    .workspace
                    .fix_file(FixFileParams {
                        path: PgLSPath::new(&workspace_file.path),
                        fix_file_mode,
                        only: only.clone(),
                        skip: skip.clone(),
                    })
                    .with_file_path_and_code(
                        workspace_file.path.display().to_string(),
                        category!("check"),
                    )?;

                if fixed.fixes_applied > 0 && fixed.code != input {
                    std::fs::write(&workspace_file.path, &fixed.code).map_err(|e| {
                        Message::from(WorkspaceError::runtime(&format!(
                            "Could not write to file: {e}"
                        )))
                    })?;
                    workspace_file
                        .guard()
                        .change_file(1, fixed.code.clone())
                        .with_file_path_and_code(
                            workspace_file.path.display().to_string(),
                            category!("check"),
                        )?;
                    input = fixed.code;
                    changed = true;
                }
                if fixed.skipped_suggested_fixes > 0 {
                    ctx.push_message(Message::SkippedFixes {
                        skipped_suggested_fixes: fixed.skipped_suggested_fixes,
                    });
                }
            }

            let max_diagnostics = ctx.remaining_diagnostics.load(Ordering::Relaxed);

            let pull_diagnostics_result = workspace_file
                .guard()
                .pull_file_diagnostics(
                    RuleCategoriesBuilder::default().all().build(),
                    max_diagnostics,
                    only.clone(),
                    skip.clone(),
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
                    skipped_diagnostics: pull_diagnostics_result.skipped_diagnostics,
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
