//! # Module
//!
//! This is where the main CLI session starts. The module is responsible
//! to parse commands and arguments, redirect the execution of the commands and
//! execute the traversal of directory and files, based on the command that was passed.

use biome_deserialize::Merge;
use cli_options::CliOptions;
use commands::check::{self, CheckArgs};
use pgls_configuration::PartialConfiguration;
use pgls_console::{ColorMode, Console, ConsoleExt, markup};
use pgls_fs::{ConfigName, FileSystem, OsFileSystem};
use pgls_workspace::{App, DynRef, Workspace, WorkspaceRef};

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
mod workspace;

use crate::cli_options::ColorsArg;
pub use crate::commands::{PgLSCommand, pg_l_s_command};
pub use crate::logging::{LoggingLevel, setup_cli_subscriber};
use crate::reporter::Report;
pub use diagnostics::CliDiagnostic;
pub use execute::{ExecutionConfig, ExecutionMode, VcsTargeting};
pub use panic::setup_panic_handler;
pub use reporter::{ReportConfig, Reporter, TraversalData};
pub use service::{SocketTransport, open_transport};

pub(crate) use pgls_env::VERSION;

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
    pub fn run(self, command: PgLSCommand) -> Result<(), CliDiagnostic> {
        let has_metrics = command.has_metrics();
        if has_metrics {
            crate::metrics::init_metrics();
        }

        let result = match command {
            PgLSCommand::Version(_) => commands::version::version(self),
            PgLSCommand::Dblint {
                cli_options,
                configuration,
            } => commands::dblint::dblint(self, &cli_options, configuration),
            PgLSCommand::Check {
                cli_options,
                configuration,
                paths,
                stdin_file_path,
                staged,
                changed,
                since,
            } => check::check(
                self,
                &cli_options,
                CheckArgs {
                    configuration,
                    paths,
                    stdin_file_path,
                    staged,
                    changed,
                    since,
                },
            ),
            PgLSCommand::Clean => commands::clean::clean(self),
            PgLSCommand::Start {
                config_path,
                log_path,
                log_prefix_name,
            } => commands::daemon::start(self, config_path, Some(log_path), Some(log_prefix_name)),
            PgLSCommand::Stop => commands::daemon::stop(self),
            PgLSCommand::Init => commands::init::init(self),
            PgLSCommand::LspProxy {
                config_path,
                log_path,
                log_prefix_name,
                ..
            } => commands::daemon::lsp_proxy(config_path, Some(log_path), Some(log_prefix_name)),
            PgLSCommand::RunServer {
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
            PgLSCommand::PrintSocket => commands::daemon::print_socket(),
        };

        if has_metrics {
            metrics::print_metrics();
        }

        result
    }

    pub fn fs(&self) -> &DynRef<'app, dyn FileSystem> {
        &self.app.fs
    }

    pub fn console(&mut self) -> &mut (dyn Console + 'app) {
        &mut *self.app.console
    }

    pub fn workspace(&self) -> &(dyn Workspace + 'app) {
        &*self.app.workspace
    }

    pub fn prepare_with_config(
        &mut self,
        cli_options: &CliOptions,
        cli_configuration: Option<PartialConfiguration>,
    ) -> Result<PartialConfiguration, CliDiagnostic> {
        setup_cli_subscriber(cli_options.log_level, cli_options.log_kind);

        let fs = self.fs();
        let loaded_configuration =
            workspace::load_config(fs, cli_options.as_configuration_path_hint())?;

        if let Some(config_path) = &loaded_configuration.file_path {
            if let Some(file_name) = config_path.file_name().and_then(|name| name.to_str()) {
                if ConfigName::is_deprecated(file_name) {
                    self.console().log(markup! {
                        <Warn>"Warning: "</Warn>
                        "Deprecated config filename detected. Use 'postgres-language-server.jsonc'.\n"
                    });
                }
            }
        }

        let mut configuration = loaded_configuration.configuration;
        if let Some(cli_config) = cli_configuration {
            configuration.merge_with(cli_config);
        }

        Ok(configuration)
    }

    pub fn setup_workspace(
        &mut self,
        configuration: PartialConfiguration,
        vcs: VcsIntegration,
    ) -> Result<(), CliDiagnostic> {
        workspace::setup_workspace(self.workspace(), self.fs(), configuration, vcs)
    }

    pub fn report(
        &mut self,
        command_name: &str,
        cli_options: &CliOptions,
        payload: &Report,
    ) -> Result<(), CliDiagnostic> {
        let mut reporter = Reporter::from_cli_options(cli_options);
        reporter.report(self.console(), command_name, payload)
    }
}

/// Controls whether workspace setup should include VCS integration details.
pub enum VcsIntegration {
    Enabled,
    Disabled,
}

pub fn to_color_mode(color: Option<&ColorsArg>) -> ColorMode {
    match color {
        Some(ColorsArg::Off) => ColorMode::Disabled,
        Some(ColorsArg::Force) => ColorMode::Enabled,
        None => ColorMode::Auto,
    }
}
