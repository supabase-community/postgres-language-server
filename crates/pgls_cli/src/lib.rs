//! # Module
//!
//! This is where the main CLI session starts. The module is responsible
//! to parse commands and arguments, redirect the execution of the commands and
//! execute the traversal of directory and files, based on the command that was passed.

use cli_options::CliOptions;
use commands::CommandRunner;
use commands::check::CheckCommandPayload;
use pgls_console::{ColorMode, Console};
use pgls_fs::OsFileSystem;
use pgls_workspace::{App, DynRef, Workspace, WorkspaceRef};
use std::env;

mod changed;
mod cli_options;
mod commands;
mod diagnostics;
mod execute;
mod logging;
mod metrics;
mod panic;
mod reporter;
mod service;

use crate::cli_options::ColorsArg;
pub use crate::commands::{PgtCommand, pgt_command};
pub use crate::logging::{LoggingLevel, setup_cli_subscriber};
pub use diagnostics::CliDiagnostic;
pub use execute::{Execution, TraversalMode, VcsTargeted, execute_mode};
pub use panic::setup_panic_handler;
pub use reporter::{DiagnosticsPayload, Reporter, ReporterVisitor, TraversalSummary};
pub use service::{SocketTransport, open_transport};

pub(crate) const VERSION: &str = match option_env!("PGT_VERSION") {
    Some(version) => version,
    None => env!("CARGO_PKG_VERSION"),
};

/// Global context for an execution of the CLI
pub struct CliSession<'app> {
    /// Instance of [App] used by this run of the CLI
    pub app: App<'app>,
}

impl<'app> CliSession<'app> {
    pub fn new(
        workspace: &'app dyn Workspace,
        console: &'app mut dyn Console,
    ) -> Result<Self, CliDiagnostic> {
        Ok(Self {
            app: App::new(
                DynRef::Owned(Box::<OsFileSystem>::default()),
                console,
                WorkspaceRef::Borrowed(workspace),
            ),
        })
    }

    /// Main function to run the CLI
    pub fn run(self, command: PgtCommand) -> Result<(), CliDiagnostic> {
        let has_metrics = command.has_metrics();
        if has_metrics {
            crate::metrics::init_metrics();
        }

        let result = match command {
            PgtCommand::Version(_) => commands::version::full_version(self),
            PgtCommand::Check {
                cli_options,
                configuration,
                paths,
                stdin_file_path,
                staged,
                changed,
                since,
            } => run_command(
                self,
                &cli_options,
                CheckCommandPayload {
                    configuration,
                    paths,
                    stdin_file_path,
                    staged,
                    changed,
                    since,
                },
            ),
            PgtCommand::Clean => commands::clean::clean(self),
            PgtCommand::Start {
                config_path,
                log_path,
                log_prefix_name,
            } => commands::daemon::start(self, config_path, Some(log_path), Some(log_prefix_name)),
            PgtCommand::Stop => commands::daemon::stop(self),
            PgtCommand::Init => commands::init::init(self),
            PgtCommand::LspProxy {
                config_path,
                log_path,
                log_prefix_name,
                ..
            } => commands::daemon::lsp_proxy(config_path, Some(log_path), Some(log_prefix_name)),
            PgtCommand::RunServer {
                stop_on_disconnect,
                config_path,
                log_path,
                log_prefix_name,
                log_level,
                log_kind,
            } => commands::daemon::run_server(
                stop_on_disconnect,
                config_path,
                Some(log_path),
                Some(log_prefix_name),
                Some(log_level),
                Some(log_kind),
            ),
            PgtCommand::PrintSocket => commands::daemon::print_socket(),
        };

        if has_metrics {
            metrics::print_metrics();
        }

        result
    }
}

pub fn to_color_mode(color: Option<&ColorsArg>) -> ColorMode {
    match color {
        Some(ColorsArg::Off) => ColorMode::Disabled,
        Some(ColorsArg::Force) => ColorMode::Enabled,
        None => ColorMode::Auto,
    }
}

pub(crate) fn run_command(
    session: CliSession,
    cli_options: &CliOptions,
    mut command: impl CommandRunner,
) -> Result<(), CliDiagnostic> {
    let command = &mut command;
    command.run(session, cli_options)
}
