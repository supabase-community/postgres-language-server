use crate::execute::StdinPayload;
use crate::execute::config::ExecutionConfig;
use crate::{CliDiagnostic, CliSession};
use pgls_console::{ConsoleExt, markup};

pub(crate) fn process(
    session: &mut CliSession,
    _config: &ExecutionConfig,
    payload: StdinPayload,
) -> Result<(), CliDiagnostic> {
    session.console().append(markup! {{payload.content}});
    Ok(())
}
