use crate::commands::daemon::default_pgls_log_path;
use crate::{CliDiagnostic, CliSession};
use pgls_env::PGLS_ENV;
use std::fs::{create_dir, remove_dir_all};
use std::path::PathBuf;

/// Runs the clean command
pub(crate) fn clean(_cli_session: CliSession) -> Result<(), CliDiagnostic> {
    let logs_path = PGLS_ENV
        .pgls_log_path
        .value()
        .map_or(default_pgls_log_path(), PathBuf::from);
    remove_dir_all(logs_path.clone()).and_then(|_| create_dir(logs_path))?;
    Ok(())
}
