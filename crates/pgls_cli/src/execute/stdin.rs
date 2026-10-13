use crate::execute::StdinPayload;
use crate::execute::config::{ExecutionConfig, ExecutionMode};
use crate::{CliDiagnostic, CliSession};
use pgls_console::{ConsoleExt, markup};
use pgls_diagnostics::{DiagnosticExt, Error, PrintDiagnostic, category};
use pgls_fs::PgLSPath;
use pgls_workspace::features::format::PullFileFormattingParams;
use pgls_workspace::workspace::{FileGuard, OpenFileParams};

pub(crate) fn process(
    session: &mut CliSession,
    config: &ExecutionConfig,
    payload: StdinPayload,
) -> Result<(), CliDiagnostic> {
    match config.mode {
        ExecutionMode::Check { .. } => {
            session.console().append(markup! {{payload.content}});
            Ok(())
        }
        ExecutionMode::Format { .. } => format(session, payload),
    }
}

/// Formats the content read from stdin and prints only the formatted code to stdout.
///
/// Diagnostics are printed to stderr. Statements that could not be formatted are kept as
/// they are, so stdout always contains the best-effort output.
fn format(session: &mut CliSession, payload: StdinPayload) -> Result<(), CliDiagnostic> {
    let pgls_path = PgLSPath::new(&payload.path);
    let workspace = session.workspace();

    let result = {
        let guard = FileGuard::open(
            workspace,
            OpenFileParams {
                path: pgls_path.clone(),
                version: 0,
                content: payload.content.clone(),
            },
        )?;
        guard.pull_file_formatting(PullFileFormattingParams {
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
