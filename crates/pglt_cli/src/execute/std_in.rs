//! In here, there are the operations that run via standard input
//!
use crate::execute::Execution;
use crate::{CliDiagnostic, CliSession};
use pglt_console::{markup, ConsoleExt};
use pglt_fs::PgLspPath;

pub(crate) fn run<'a>(
    session: CliSession,
    mode: &'a Execution,
    pglsp_path: PgLspPath,
    content: &'a str,
    verbose: bool,
) -> Result<(), CliDiagnostic> {
    let workspace = &*session.app.workspace;
    let console = &mut *session.app.console;
    let version = 0;

    console.append(markup! {{content}});
    Ok(())
}
