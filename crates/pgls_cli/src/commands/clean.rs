use crate::CliDiagnostic;
use std::fs::{create_dir, remove_dir_all};
use std::path::PathBuf;

/// Runs the clean command
pub(crate) fn clean() -> Result<(), CliDiagnostic> {
    let logs_path = pgls_env::PGLS_LOG_PATH.value().map_or_else(
        || pgls_fs::ensure_cache_dir().join("pgls-logs"),
        PathBuf::from,
    );
    remove_dir_all(&logs_path).and_then(|()| create_dir(&logs_path))?;
    Ok(())
}
