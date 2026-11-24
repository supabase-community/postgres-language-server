use crate::CliDiagnostic;
use crate::commands::LogOptions;
use std::fs::{create_dir, remove_dir_all};

/// Runs the clean command: empties the folder of the logs.
pub(crate) fn clean(log_options: &LogOptions) -> Result<(), CliDiagnostic> {
    let logs_path = &log_options.log_path;
    remove_dir_all(logs_path).and_then(|()| create_dir(logs_path))?;
    Ok(())
}
