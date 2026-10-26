//! # Module
//!
//! This is where the main CLI session starts. The module is responsible
//! to parse commands and arguments, redirect the execution of the commands and
//! execute the traversal of directory and files, based on the command that was passed.

use cli_options::CliOptions;
use commands::check::{self, CheckArgs};
use commands::format::{self, FormatArgs};
use pgls_configuration::Merge;
use pgls_configuration::PartialConfiguration;
use pgls_configuration::database::PartialDatabaseConfiguration;
use pgls_console::{ColorMode, Console, ConsoleExt, markup};
use pgls_diagnostics::{DiagnosticExt, Error};
use pgls_fs::{ConfigName, FileSystem, OsFileSystem};
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::Arc;

use pgls_workspace::configuration::{
    NestedConfiguration, NestedConfigurationSearch, apply_nested_configurations,
};
use pgls_workspace::settings::FilesSettings;
use pgls_workspace::workspace::ProjectKey;
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

pub(crate) use pgls_workspace::WORKER_STACK_SIZE_BYTES;

/// A multi-threaded tokio runtime whose threads have a stack of [`WORKER_STACK_SIZE_BYTES`]. It
/// runs the language server's handlers and their `spawn_blocking` threads.
pub(crate) fn runtime() -> std::io::Result<tokio::runtime::Runtime> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_stack_size(WORKER_STACK_SIZE_BYTES)
        .build()
}

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
    /// The project of the working directory, once [CliSession::setup_workspace] registered it
    project_key: ProjectKey,
    /// The files settings of the root configuration, which decide the folders to search for
    /// nested configuration files
    root_files: Option<FilesSettings>,
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
            project_key: ProjectKey::default(),
            root_files: None,
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
                fix,
                unsafe_,
                only,
                skip,
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
                    fix,
                    unsafe_,
                    only,
                    skip,
                },
            ),
            PgLSCommand::Format {
                cli_options,
                configuration,
                paths,
                write,
                stdin_file_path,
                staged,
                changed,
                since,
            } => format::format(
                self,
                &cli_options,
                FormatArgs {
                    configuration,
                    paths,
                    write,
                    stdin_file_path,
                    staged,
                    changed,
                    since,
                },
            ),
            PgLSCommand::Parse {
                file,
                stdin_file_path,
            } => {
                let path = file.or_else(|| stdin_file_path.as_deref().map(PathBuf::from));
                let input = commands::parse::read_input(path.as_deref())?;
                commands::parse::run_parse(&input, path.as_deref())
            }
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
            PgLSCommand::SchemaExport {
                connection_string,
                output,
            } => {
                let connection_string = connection_string
                    .or_else(|| std::env::var("DATABASE_URL").ok())
                    .ok_or_else(|| {
                        CliDiagnostic::missing_argument(
                            "--connection-string (or DATABASE_URL env var)",
                            "schema-export",
                        )
                    })?;
                let runtime = runtime().map_err(CliDiagnostic::io_error)?;
                runtime.block_on(commands::schema_export::run_schema_export(
                    &connection_string,
                    output.as_deref(),
                ))
            }
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

        if let Some(config_path) = &loaded_configuration.file_path
            && let Some(file_name) = config_path.file_name().and_then(|name| name.to_str())
            && ConfigName::is_deprecated(file_name)
        {
            self.console().log(markup! {
                <Warn>"Warning: "</Warn>
                "Deprecated config filename detected. Use 'postgres-language-server.jsonc'.\n"
            });
        }

        if loaded_configuration.is_user_configuration
            && let Some(file_path) = &loaded_configuration.file_path
        {
            // On stderr, so that it doesn't end up in the output of a reporter or of `format`.
            self.console().error(markup! {
                <Info>"Using the user configuration at "{file_path.display().to_string()}"\n"</Info>
            });
        }

        for deprecation in loaded_configuration.configuration.deprecations() {
            self.console().log(markup! {
                <Warn>"Warning: "</Warn>{deprecation}"\n"
            });
        }

        Ok(apply_overrides(
            loaded_configuration.configuration,
            cli_configuration,
        ))
    }

    /// The project of the working directory, which every file of this run belongs to
    pub fn project_key(&self) -> ProjectKey {
        self.project_key
    }

    /// Applies the configuration files that apply to `paths` to the project, so that each file
    /// uses its nearest configuration file below the working directory. The arguments apply to
    /// them like to the root configuration file, and the environment fills in the connection
    /// options they don't set. Skipped when the user chose a configuration file.
    ///
    /// The folder of a configuration file that can't be loaded uses the default settings. Returns
    /// an error diagnostic for each such file.
    pub fn apply_nested_configurations(
        &mut self,
        cli_options: &CliOptions,
        cli_configuration: Option<PartialConfiguration>,
        paths: &[OsString],
    ) -> Result<Vec<Error>, CliDiagnostic> {
        if cli_options.config_path.is_some() {
            return Ok(Vec::new());
        }
        let Some(root) = self.fs().working_directory() else {
            return Ok(Vec::new());
        };

        let root_files = self.root_files.take().map(Arc::new);
        let nested = apply_nested_configurations(
            self.workspace(),
            self.fs(),
            self.project_key,
            NestedConfigurationSearch {
                paths: Some(paths.iter().map(PathBuf::from).collect()),
                root_files,
                ..NestedConfigurationSearch::new(root)
            },
            PartialDatabaseConfiguration::from_env().as_ref(),
            |configuration| apply_arguments(configuration, cli_configuration.clone()),
        )?;
        if let Some(incomplete) = nested.incomplete {
            self.console().log(markup! {
                <Warn>"Warning: "</Warn>{incomplete.to_string()}"\n"
            });
        }
        let errors = nested
            .configurations
            .into_iter()
            .filter_map(
                |NestedConfiguration {
                     directory, result, ..
                 }| {
                    let error = result.err()?;
                    let file_path = ConfigName::file_names()
                        .iter()
                        .map(|file_name| directory.join(file_name))
                        .find(|file_path| self.fs().path_is_file(file_path))
                        .unwrap_or(directory);
                    Some(Error::from(error).with_file_path(file_path.display().to_string()))
                },
            )
            .collect();
        Ok(errors)
    }

    pub fn setup_workspace(
        &mut self,
        configuration: PartialConfiguration,
        vcs: VcsIntegration,
    ) -> Result<(), CliDiagnostic> {
        let (project_key, root_files) =
            workspace::setup_workspace(self.workspace(), self.fs(), configuration, vcs)?;
        self.project_key = project_key;
        self.root_files = Some(root_files);
        Ok(())
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

/// Lays the database configuration from the environment and then the arguments over a
/// configuration file.
fn apply_overrides(
    mut configuration: PartialConfiguration,
    cli_configuration: Option<PartialConfiguration>,
) -> PartialConfiguration {
    // Env vars override config file but are overridden by explicit CLI args.
    if let Some(env_db) = PartialDatabaseConfiguration::from_env() {
        let env_config = PartialConfiguration {
            db: Some(env_db),
            ..Default::default()
        };
        configuration.merge_with(env_config);
    }

    apply_arguments(configuration, cli_configuration)
}

/// Lays the configuration from the arguments over a configuration file.
fn apply_arguments(
    mut configuration: PartialConfiguration,
    cli_configuration: Option<PartialConfiguration>,
) -> PartialConfiguration {
    if let Some(cli_config) = cli_configuration {
        configuration.merge_with(cli_config);
    }

    configuration
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

#[cfg(test)]
mod runtime_tests {
    /// A long expression needs more stack than the 2 MiB default of tokio's threads. In a debug
    /// build, 2,000 terms already overflow 2 MiB.
    #[test]
    fn workers_parse_long_expressions() {
        let sql = format!("select 1{};", " + abs(1)".repeat(2_000));
        let runtime = super::runtime().unwrap();
        let parsed = runtime
            .block_on(runtime.spawn(async move { pgls_query::parse(&sql).is_ok() }))
            .unwrap();
        // The tree is too deep for the protobuf decoder, so it doesn't parse, but the parser
        // returns instead of overflowing the stack.
        assert!(!parsed);
    }
}
