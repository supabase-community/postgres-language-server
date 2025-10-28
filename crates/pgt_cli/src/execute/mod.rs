mod config;
mod diagnostics;
mod process_file;
mod stdin;
mod walk;

pub use config::{ExecutionConfig, ExecutionMode, VcsTargeting};

use crate::reporter::Report;
use crate::{CliDiagnostic, CliSession};
use std::ffi::OsString;
use std::path::PathBuf;

pub fn run_files(
    session: &mut CliSession,
    config: &ExecutionConfig,
    paths: Vec<OsString>,
) -> Result<Report, CliDiagnostic> {
    walk::traverse(session, config, paths)
}

pub struct StdinPayload {
    #[allow(dead_code)]
    pub path: PathBuf,
    pub content: String,
}

pub fn run_stdin(
    session: &mut CliSession,
    config: &ExecutionConfig,
    payload: StdinPayload,
) -> Result<(), CliDiagnostic> {
    stdin::process(session, config, payload)
}
