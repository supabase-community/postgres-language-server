use crate::execute::StdinPayload;
use crate::execute::config::{ExecutionConfig, ExecutionMode};
use crate::execute::diagnostics::SkippedDiagnostic;
use crate::reporter::Report;
use crate::{CliDiagnostic, CliSession};
use pgls_analyse::RuleCategoriesBuilder;
use pgls_configuration::RuleSelector;
use pgls_console::{Console, ConsoleExt, markup};
use pgls_diagnostics::{DiagnosticExt, Error, PrintDiagnostic};
use pgls_fs::PgLSPath;
use pgls_workspace::features::code_actions::{FixFileMode, FixFileParams};
use pgls_workspace::features::format::PullFileFormattingParams;
use pgls_workspace::workspace::{FileGuard, OpenFileParams};
use std::time::Instant;

pub(crate) fn process(
    session: &mut CliSession,
    config: &ExecutionConfig,
    payload: StdinPayload,
) -> Result<Report, CliDiagnostic> {
    let start = Instant::now();
    let diagnostics = match &config.mode {
        ExecutionMode::Check {
            fix, only, skip, ..
        } => check(session, config, payload, *fix, only.clone(), skip.clone()),
        ExecutionMode::Format { .. } => format(session, config, payload),
    }?;
    Ok(Report::new(diagnostics, start.elapsed(), 0, None))
}

/// Checks the content read from stdin and prints it to stdout, with the fixes applied with
/// `--fix`. Returns the diagnostics, which it prints to stderr.
fn check(
    session: &mut CliSession,
    config: &ExecutionConfig,
    payload: StdinPayload,
    fix: Option<FixFileMode>,
    only: Vec<RuleSelector>,
    skip: Vec<RuleSelector>,
) -> Result<Vec<Error>, CliDiagnostic> {
    // The settings of the project apply to absolute paths, like those of the traversal.
    let path = session
        .fs()
        .working_directory()
        .map_or_else(|| payload.path.clone(), |cwd| cwd.join(&payload.path));
    let pgls_path = PgLSPath::new(&path);
    let project_key = session.project_key();
    let workspace = session.workspace();

    let checked = {
        let guard = FileGuard::open(
            workspace,
            OpenFileParams {
                project_key,
                path: pgls_path.clone(),
                version: 0,
                content: payload.content.clone(),
            },
        )?;
        let pull_diagnostics = || {
            guard.pull_file_diagnostics(
                RuleCategoriesBuilder::default().all().build(),
                config.max_diagnostics(),
                only.clone(),
                skip.clone(),
            )
        };
        // `--skip-errors` prints a file with syntax errors as it is, so it looks for them before
        // the fixes.
        if config.skip_errors && fix.is_some() && config.skips(&pull_diagnostics()?.diagnostics) {
            None
        } else {
            let mut content = payload.content.clone();
            if let Some(fix_file_mode) = fix {
                let fixed = workspace.fix_file(FixFileParams {
                    project_key,
                    path: pgls_path.clone(),
                    fix_file_mode,
                    only: only.clone(),
                    skip: skip.clone(),
                })?;
                if fixed.code != content {
                    guard.change_file(1, fixed.code.clone())?;
                    content = fixed.code;
                }
            }
            let result = pull_diagnostics()?;
            (!config.skips(&result.diagnostics)).then_some((content, result.diagnostics))
        }
    };

    let Some((content, diagnostics)) = checked else {
        return Ok(payload.print_skipped(session.console()));
    };

    let console = session.console();
    console.append(markup! {{content}});

    Ok(print_diagnostics(
        console,
        diagnostics,
        &payload.path.display().to_string(),
        &content,
    ))
}

/// Formats the content read from stdin and prints only the formatted code to stdout.
///
/// Returns the diagnostics, which it prints to stderr. Statements that could not be formatted are
/// kept as they are, so stdout always contains the best-effort output.
fn format(
    session: &mut CliSession,
    config: &ExecutionConfig,
    payload: StdinPayload,
) -> Result<Vec<Error>, CliDiagnostic> {
    let pgls_path = PgLSPath::new(&payload.path);
    let project_key = session.project_key();
    let workspace = session.workspace();

    let result = {
        let guard = FileGuard::open(
            workspace,
            OpenFileParams {
                project_key,
                path: pgls_path.clone(),
                version: 0,
                content: payload.content.clone(),
            },
        )?;
        guard.pull_file_formatting(PullFileFormattingParams {
            project_key,
            path: pgls_path,
            range: None,
        })?
    };

    if config.skips(&result.diagnostics) {
        return Ok(payload.print_skipped(session.console()));
    }

    // The workspace returns an empty result for an ignored file or a disabled formatter.
    let output = if result.original == payload.content {
        result.formatted
    } else {
        payload.content.clone()
    };

    let console = session.console();
    console.append(markup! {{output}});

    Ok(print_diagnostics(
        console,
        result.diagnostics,
        &payload.path.display().to_string(),
        &payload.content,
    ))
}

impl StdinPayload {
    /// Skips the code for `--skip-errors` because of its syntax errors: prints it to stdout as it
    /// is, and reports that it was skipped instead of its diagnostics.
    fn print_skipped(&self, console: &mut dyn Console) -> Vec<Error> {
        console.append(markup! {{self.content}});
        print_diagnostics(
            console,
            [SkippedDiagnostic],
            &self.path.display().to_string(),
            &self.content,
        )
    }
}

/// Prints the diagnostics of the code read from stdin to stderr, as stdout gets the code.
fn print_diagnostics(
    console: &mut dyn Console,
    diagnostics: impl IntoIterator<Item = impl Into<Error>>,
    file_name: &str,
    content: &str,
) -> Vec<Error> {
    diagnostics
        .into_iter()
        .map(|diagnostic| {
            let diagnostic = diagnostic
                .into()
                .with_file_path(file_name)
                .with_file_source_code(content);
            console.error(markup! {{PrintDiagnostic::simple(&diagnostic)}});
            diagnostic
        })
        .collect()
}
