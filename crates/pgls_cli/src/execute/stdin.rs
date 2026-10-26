use crate::execute::StdinPayload;
use crate::execute::config::{ExecutionConfig, ExecutionMode};
use crate::{CliDiagnostic, CliSession};
use pgls_analyse::RuleCategoriesBuilder;
use pgls_configuration::RuleSelector;
use pgls_console::{ConsoleExt, markup};
use pgls_diagnostics::{DiagnosticExt, Error, PrintDiagnostic, Severity, category};
use pgls_fs::PgLSPath;
use pgls_workspace::features::code_actions::{FixFileMode, FixFileParams};
use pgls_workspace::features::format::PullFileFormattingParams;
use pgls_workspace::workspace::{FileGuard, OpenFileParams};

pub(crate) fn process(
    session: &mut CliSession,
    config: &ExecutionConfig,
    payload: StdinPayload,
) -> Result<(), CliDiagnostic> {
    match &config.mode {
        ExecutionMode::Check {
            fix, only, skip, ..
        } => check(session, config, payload, *fix, only.clone(), skip.clone()),
        ExecutionMode::Format { .. } => format(session, payload),
    }
}

/// Checks the content read from stdin and prints it to stdout, with the fixes applied with
/// `--fix`. Diagnostics are printed to stderr, and errors among them fail the command.
fn check(
    session: &mut CliSession,
    config: &ExecutionConfig,
    payload: StdinPayload,
    fix: Option<FixFileMode>,
    only: Vec<RuleSelector>,
    skip: Vec<RuleSelector>,
) -> Result<(), CliDiagnostic> {
    // The settings of the project apply to absolute paths, like those of the traversal.
    let path = session
        .fs()
        .working_directory()
        .map_or_else(|| payload.path.clone(), |cwd| cwd.join(&payload.path));
    let pgls_path = PgLSPath::new(&path);
    let project_key = session.project_key();
    let workspace = session.workspace();

    let (content, diagnostics) = {
        let guard = FileGuard::open(
            workspace,
            OpenFileParams {
                project_key,
                path: pgls_path.clone(),
                version: 0,
                content: payload.content.clone(),
            },
        )?;
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
        let result = guard.pull_file_diagnostics(
            RuleCategoriesBuilder::default().all().build(),
            config.max_diagnostics(),
            only,
            skip,
        )?;
        (content, result.diagnostics)
    };

    let console = session.console();
    console.append(markup! {{content}});

    let file_name = payload.path.display().to_string();
    let mut has_errors = false;
    for diagnostic in diagnostics {
        let diagnostic = Error::from(diagnostic)
            .with_file_path(&file_name)
            .with_file_source_code(&content);
        has_errors |= diagnostic.severity() >= Severity::Error;
        console.error(markup! {{PrintDiagnostic::simple(&diagnostic)}});
    }

    if has_errors {
        Err(CliDiagnostic::check_error(category!("check")))
    } else {
        Ok(())
    }
}

/// Formats the content read from stdin and prints only the formatted code to stdout.
///
/// Diagnostics are printed to stderr. Statements that could not be formatted are kept as
/// they are, so stdout always contains the best-effort output.
fn format(session: &mut CliSession, payload: StdinPayload) -> Result<(), CliDiagnostic> {
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

    // The workspace returns an empty result for an ignored file or a disabled formatter.
    let output = if result.original == payload.content {
        result.formatted
    } else {
        payload.content.clone()
    };

    let console = session.console();
    console.append(markup! {{output}});

    if result.diagnostics.is_empty() {
        return Ok(());
    }

    let file_name = payload.path.display().to_string();
    for diagnostic in result.diagnostics {
        let diagnostic = Error::from(diagnostic)
            .with_file_path(&file_name)
            .with_file_source_code(&payload.content);
        console.error(markup! {{PrintDiagnostic::simple(&diagnostic)}});
    }

    Err(CliDiagnostic::check_error(category!("format")))
}
