use crate::diagnostics::FormatDiffDiagnostic;
use crate::execute::process_file::workspace_file::WorkspaceFile;
use crate::execute::process_file::{FileResult, FileStatus, Message, SharedTraversalOptions};
use pgls_workspace::features::format::PullFileFormattingParams;
use std::io::Write;
use std::path::Path;

pub(crate) fn format_file<'ctx>(ctx: &SharedTraversalOptions<'ctx, '_>, path: &Path) -> FileResult {
    let mut workspace_file = WorkspaceFile::new(ctx, path)?;
    format_with_guard(ctx, &mut workspace_file)
}

fn format_with_guard<'ctx>(
    ctx: &SharedTraversalOptions<'ctx, '_>,
    workspace_file: &mut WorkspaceFile,
) -> FileResult {
    let path = workspace_file.path.clone();
    let pgls_path = pgls_fs::PgLSPath::new(&path);

    let guard = workspace_file.guard();
    let format_result = guard.pull_file_formatting(PullFileFormattingParams {
        project_key: guard.project_key(),
        path: pgls_path.clone(),
        range: None,
    });

    let format_result = format_result.map_err(Message::from)?;

    let mut diagnostics: Vec<pgls_diagnostics::Error> = format_result
        .diagnostics
        .into_iter()
        .map(pgls_diagnostics::Error::from)
        .collect();

    if format_result.original == format_result.formatted && diagnostics.is_empty() {
        return Ok(FileStatus::Unchanged);
    }

    if ctx.config.allows_writes() {
        let file_path = path.to_path_buf();
        let mut file = std::fs::File::create(&file_path).map_err(|e| {
            Message::from(pgls_workspace::WorkspaceError::runtime(&format!(
                "Could not write to file: {e}"
            )))
        })?;
        file.write_all(format_result.formatted.as_bytes())
            .map_err(|e| {
                Message::from(pgls_workspace::WorkspaceError::runtime(&format!(
                    "Could not write to file: {e}"
                )))
            })?;

        if !diagnostics.is_empty() {
            ctx.push_message(Message::Diagnostics {
                name: path.display().to_string(),
                content: format_result.original.clone(),
                diagnostics,
                skipped_diagnostics: 0,
            });
        }

        return Ok(FileStatus::Changed);
    }

    let only_layout_between_statements = format_result.statements.is_empty();

    for stmt in format_result.statements {
        let diff = pgls_text_edit::TextEdit::from_unicode_words(&stmt.original, &stmt.formatted);
        let start_byte: usize = stmt.range.start().into();
        let start_line = format_result.original[..start_byte]
            .chars()
            .filter(|&c| c == '\n')
            .count() as u32;
        diagnostics.push(pgls_diagnostics::Error::from(FormatDiffDiagnostic {
            file_name: path.display().to_string(),
            diff,
            start_line,
        }));
    }

    // Only the blank lines between statements change: show the whole file.
    if only_layout_between_statements && format_result.original != format_result.formatted {
        diagnostics.push(pgls_diagnostics::Error::from(FormatDiffDiagnostic {
            file_name: path.display().to_string(),
            diff: pgls_text_edit::TextEdit::from_unicode_words(
                &format_result.original,
                &format_result.formatted,
            ),
            start_line: 0,
        }));
    }

    if diagnostics.is_empty() {
        Ok(FileStatus::Unchanged)
    } else {
        Ok(FileStatus::Message(Message::Diagnostics {
            name: path.display().to_string(),
            content: format_result.original,
            diagnostics,
            skipped_diagnostics: 0,
        }))
    }
}
