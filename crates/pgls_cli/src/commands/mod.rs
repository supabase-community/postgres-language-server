use crate::changed::{get_changed_files, get_staged_files};
use crate::cli_options::{CliOptions, CliReporter, ColorsArg, cli_options};
use crate::execute::Stdin;
use crate::logging::LoggingKind;
use crate::{
    CliDiagnostic, CliSession, Execution, LoggingLevel, VERSION, execute_mode, setup_cli_subscriber,
};
use bpaf::Bpaf;
use pgls_configuration::{PartialConfiguration, partial_configuration};
use pgls_console::{Console, ConsoleExt, markup};
use pgls_fs::{ConfigName, FileSystem};
use pgls_workspace::PartialConfigurationExt;
use pgls_workspace::configuration::{LoadedConfiguration, load_configuration};
use pgls_workspace::workspace::{RegisterProjectFolderParams, UpdateSettingsParams};
use pgls_workspace::{DynRef, Workspace, WorkspaceError};
use std::ffi::OsString;
use std::path::PathBuf;
pub(crate) mod check;
pub(crate) mod clean;
pub(crate) mod daemon;
pub(crate) mod init;
pub(crate) mod version;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options, version(VERSION))]
#[allow(clippy::large_enum_variant)]
/// Postgres Tools official CLI. Use it to check the health of your project or run it to check single files.
pub enum PgtCommand {
    /// Shows the version information and quit.
    #[bpaf(command)]
    Version(#[bpaf(external(cli_options), hide_usage)] CliOptions),

    /// Runs everything to the requested files.
    #[bpaf(command)]
    Check {
        #[bpaf(external(partial_configuration), hide_usage, optional)]
        configuration: Option<PartialConfiguration>,

        #[bpaf(external, hide_usage)]
        cli_options: CliOptions,

        /// Use this option when you want to format code piped from `stdin`, and print the output to `stdout`.
        ///
        /// The file doesn't need to exist on disk, what matters is the extension of the file. Based on the extension, we know how to check the code.
        ///
        /// Example: `echo 'let a;' | pgls_cli check --stdin-file-path=test.sql`
        #[bpaf(long("stdin-file-path"), argument("PATH"), hide_usage)]
        stdin_file_path: Option<String>,

        /// When set to true, only the files that have been staged (the ones prepared to be committed)
        /// will be linted. This option should be used when working locally.
        #[bpaf(long("staged"), switch)]
        staged: bool,

        /// When set to true, only the files that have been changed compared to your `defaultBranch`
        /// configuration will be linted. This option should be used in CI environments.
        #[bpaf(long("changed"), switch)]
        changed: bool,

        /// Use this to specify the base branch to compare against when you're using the --changed
        /// flag and the `defaultBranch` is not set in your `postgres-language-server.jsonc`
        #[bpaf(long("since"), argument("REF"))]
        since: Option<String>,

        /// Single file, single path or list of paths
        #[bpaf(positional("PATH"), many)]
        paths: Vec<OsString>,
    },

    /// Starts the daemon server process.
    #[bpaf(command)]
    Start {
        /// Allows to change the prefix applied to the file name of the logs.
        #[bpaf(
            env("PGT_LOG_PREFIX_NAME"),
            env("PGLS_LOG_PREFIX_NAME"),
            long("log-prefix-name"),
            argument("STRING"),
            hide_usage,
            fallback(String::from("server.log")),
            display_fallback
        )]
        log_prefix_name: String,

        /// Allows to change the folder where logs are stored.
        #[bpaf(
            env("PGT_LOG_PATH"),
            env("PGLS_LOG_PATH"),
            long("log-path"),
            argument("PATH"),
            hide_usage,
            fallback(pgls_fs::ensure_cache_dir().join("pgt-logs")),
        )]
        log_path: PathBuf,
        /// Allows to set a custom file path to the configuration file,
        /// or a custom directory path to find `postgres-language-server.jsonc`
        #[bpaf(
            env("PGT_LOG_PREFIX_NAME"),
            env("PGLS_LOG_PREFIX_NAME"),
            long("config-path"),
            argument("PATH")
        )]
        config_path: Option<PathBuf>,
    },

    /// Stops the daemon server process.
    #[bpaf(command)]
    Stop,

    /// Bootstraps a new project. Creates a configuration file with some defaults.
    #[bpaf(command)]
    Init,

    /// Acts as a server for the Language Server Protocol over stdin/stdout.
    #[bpaf(command("lsp-proxy"))]
    LspProxy {
        /// Allows to change the prefix applied to the file name of the logs.
        #[bpaf(
            env("PGT_LOG_PREFIX_NAME"),
            env("PGLS_LOG_PREFIX_NAME"),
            long("log-prefix-name"),
            argument("STRING"),
            hide_usage,
            fallback(String::from("server.log")),
            display_fallback
        )]
        log_prefix_name: String,
        /// Allows to change the folder where logs are stored.
        #[bpaf(
            env("PGT_LOG_PATH"),
            env("PGLS_LOG_PATH"),
            long("log-path"),
            argument("PATH"),
            hide_usage,
            fallback(pgls_fs::ensure_cache_dir().join("pgt-logs")),
        )]
        log_path: PathBuf,
        /// Allows to set a custom file path to the configuration file,
        /// or a custom directory path to find `postgres-language-server.jsonc`
        #[bpaf(
            env("PGT_CONFIG_PATH"),
            env("PGLS_CONFIG_PATH"),
            long("config-path"),
            argument("PATH")
        )]
        config_path: Option<PathBuf>,
        /// Bogus argument to make the command work with vscode-languageclient
        #[bpaf(long("stdio"), hide, hide_usage, switch)]
        stdio: bool,
    },

    #[bpaf(command)]
    /// Cleans the logs emitted by the daemon.
    Clean,

    #[bpaf(command("__run_server"), hide)]
    RunServer {
        /// Allows to change the prefix applied to the file name of the logs.
        #[bpaf(
            env("PGT_LOG_PREFIX_NAME"),
            env("PGLS_LOG_PREFIX_NAME"),
            long("log-prefix-name"),
            argument("STRING"),
            hide_usage,
            fallback(String::from("server.log")),
            display_fallback
        )]
        log_prefix_name: String,

        /// Allows to change the folder where logs are stored.
        #[bpaf(
            env("PGT_LOG_PATH"),
            env("PGLS_LOG_PATH"),
            long("log-path"),
            argument("PATH"),
            hide_usage,
            fallback(pgls_fs::ensure_cache_dir().join("pgt-logs")),
        )]
        log_path: PathBuf,

        /// Allows to change the log level. Default is debug. This will only affect "pgt*" crates. All others are logged with info level.
        #[bpaf(
            env("PGT_LOG_LEVEL"),
            env("PGLS_LOG_LEVEL"),
            long("log-level"),
            argument("trace|debug|info|warn|error|none"),
            fallback(String::from("debug"))
        )]
        log_level: String,

        /// Allows to change the logging format kind. Default is hierarchical.
        #[bpaf(
            env("PGT_LOG_KIND"),
            env("PGLS_LOG_KIND"),
            long("log-kind"),
            argument("hierarchical|bunyan"),
            fallback(String::from("hierarchical"))
        )]
        log_kind: String,

        #[bpaf(long("stop-on-disconnect"), hide_usage)]
        stop_on_disconnect: bool,
        /// Allows to set a custom file path to the configuration file,
        /// or a custom directory path to find `postgres-language-server.jsonc`
        #[bpaf(
            env("PGT_CONFIG_PATH"),
            env("PGLS_CONFIG_PATH"),
            long("config-path"),
            argument("PATH")
        )]
        config_path: Option<PathBuf>,
    },
    #[bpaf(command("__print_socket"), hide)]
    PrintSocket,
}

impl PgtCommand {
    const fn cli_options(&self) -> Option<&CliOptions> {
        match self {
            PgtCommand::Version(cli_options) | PgtCommand::Check { cli_options, .. } => {
                Some(cli_options)
            }
            PgtCommand::LspProxy { .. }
            | PgtCommand::Start { .. }
            | PgtCommand::Stop
            | PgtCommand::Init
            | PgtCommand::RunServer { .. }
            | PgtCommand::Clean
            | PgtCommand::PrintSocket => None,
        }
    }

    pub const fn get_color(&self) -> Option<&ColorsArg> {
        match self.cli_options() {
            Some(cli_options) => {
                // To properly display GitHub annotations we need to disable colors
                if matches!(cli_options.reporter, CliReporter::GitHub) {
                    return Some(&ColorsArg::Off);
                }
                // We want force colors in CI, to give e better UX experience
                // Unless users explicitly set the colors flag
                // if matches!(self, Postgres ToolsCommand::Ci { .. }) && cli_options.colors.is_none() {
                //     return Some(&ColorsArg::Force);
                // }
                // Normal behaviors
                cli_options.colors.as_ref()
            }
            None => None,
        }
    }

    pub const fn should_use_server(&self) -> bool {
        match self.cli_options() {
            Some(cli_options) => cli_options.use_server,
            None => false,
        }
    }

    pub const fn has_metrics(&self) -> bool {
        false
    }

    pub fn is_verbose(&self) -> bool {
        self.cli_options()
            .is_some_and(|cli_options| cli_options.verbose)
    }

    pub fn log_level(&self) -> LoggingLevel {
        self.cli_options()
            .map_or(LoggingLevel::default(), |cli_options| cli_options.log_level)
    }

    pub fn log_kind(&self) -> LoggingKind {
        self.cli_options()
            .map_or(LoggingKind::default(), |cli_options| cli_options.log_kind)
    }
}

/// Generic interface for executing commands.
///
/// Consumers must implement the following methods:
///
/// - [CommandRunner::merge_configuration]
/// - [CommandRunner::get_files_to_process]
/// - [CommandRunner::get_stdin_file_path]
/// - [CommandRunner::should_write]
/// - [CommandRunner::get_execution]
///
/// Optional methods:
/// - [CommandRunner::check_incompatible_arguments]
pub(crate) trait CommandRunner: Sized {
    const COMMAND_NAME: &'static str;

    /// The main command to use.
    fn run(&mut self, session: CliSession, cli_options: &CliOptions) -> Result<(), CliDiagnostic> {
        setup_cli_subscriber(cli_options.log_level, cli_options.log_kind);
        let fs = &session.app.fs;
        let console = &mut *session.app.console;
        let workspace = &*session.app.workspace;
        self.check_incompatible_arguments()?;
        let (execution, paths) = self.configure_workspace(fs, console, workspace, cli_options)?;
        execute_mode(execution, session, cli_options, paths)
    }

    /// This function prepares the workspace with the following:
    /// - Loading the configuration file.
    /// - Configure the VCS integration
    /// - Computes the paths to traverse/handle. This changes based on the VCS arguments that were passed.
    /// - Register a project folder using the working directory.
    /// - Updates the settings that belong to the project registered
    fn configure_workspace(
        &mut self,
        fs: &DynRef<'_, dyn FileSystem>,
        console: &mut dyn Console,
        workspace: &dyn Workspace,
        cli_options: &CliOptions,
    ) -> Result<(Execution, Vec<OsString>), CliDiagnostic> {
        let loaded_configuration =
            load_configuration(fs, cli_options.as_configuration_path_hint())?;

        // Check for deprecated config filename
        if let Some(config_path) = &loaded_configuration.file_path {
            if let Some(file_name) = config_path.file_name().and_then(|n| n.to_str()) {
                if ConfigName::is_deprecated(file_name) {
                    console.log(markup! {
                        <Warn>"Warning: "</Warn>"You are using the deprecated config filename '"<Emphasis>"postgrestools.jsonc"</Emphasis>"'. \
                        Please rename it to '"<Emphasis>"postgres-language-server.jsonc"</Emphasis>"'. \
                        Support for the old filename will be removed in a future version.\n"
                    });
                }
            }
        }

        let configuration_path = loaded_configuration.directory_path.clone();
        let configuration = self.merge_configuration(loaded_configuration, fs, console)?;
        let vcs_base_path = configuration_path.or(fs.working_directory());
        let (vcs_base_path, gitignore_matches) =
            configuration.retrieve_gitignore_matches(fs, vcs_base_path.as_deref())?;
        let paths = self.get_files_to_process(fs, &configuration)?;
        workspace.register_project_folder(RegisterProjectFolderParams {
            path: fs.working_directory(),
            set_as_current_workspace: true,
        })?;

        workspace.update_settings(UpdateSettingsParams {
            workspace_directory: fs.working_directory(),
            configuration,
            vcs_base_path,
            gitignore_matches,
        })?;

        let execution = self.get_execution(cli_options, console, workspace)?;
        Ok((execution, paths))
    }

    /// Computes [Stdin] if the CLI has the necessary information.
    ///
    /// ## Errors
    /// - If the user didn't provide anything via `stdin` but the option `--stdin-file-path` is passed.
    fn get_stdin(&self, console: &mut dyn Console) -> Result<Option<Stdin>, CliDiagnostic> {
        let stdin = if let Some(stdin_file_path) = self.get_stdin_file_path() {
            let input_code = console.read();
            if let Some(input_code) = input_code {
                let path = PathBuf::from(stdin_file_path);
                Some((path, input_code).into())
            } else {
                // we provided the argument without a piped stdin, we bail
                return Err(CliDiagnostic::missing_argument("stdin", Self::COMMAND_NAME));
            }
        } else {
            None
        };

        Ok(stdin)
    }

    // Below, the methods that consumers must implement.

    /// Implements this method if you need to merge CLI arguments to the loaded configuration.
    ///
    /// The CLI arguments take precedence over the option configured in the configuration file.
    fn merge_configuration(
        &mut self,
        loaded_configuration: LoadedConfiguration,
        fs: &DynRef<'_, dyn FileSystem>,
        console: &mut dyn Console,
    ) -> Result<PartialConfiguration, WorkspaceError>;

    /// It returns the paths that need to be handled/traversed.
    fn get_files_to_process(
        &self,
        fs: &DynRef<'_, dyn FileSystem>,
        configuration: &PartialConfiguration,
    ) -> Result<Vec<OsString>, CliDiagnostic>;

    /// It returns the file path to use in `stdin` mode.
    fn get_stdin_file_path(&self) -> Option<&str>;

    /// Returns the [Execution] mode.
    fn get_execution(
        &self,
        cli_options: &CliOptions,
        console: &mut dyn Console,
        workspace: &dyn Workspace,
    ) -> Result<Execution, CliDiagnostic>;

    // Below, methods that consumers can implement

    /// Optional method that can be implemented to check if some CLI arguments aren't compatible.
    ///
    /// The method is called before loading the configuration from disk.
    fn check_incompatible_arguments(&self) -> Result<(), CliDiagnostic> {
        Ok(())
    }
}

fn get_files_to_process_with_cli_options(
    since: Option<&str>,
    changed: bool,
    staged: bool,
    fs: &DynRef<'_, dyn FileSystem>,
    configuration: &PartialConfiguration,
) -> Result<Option<Vec<OsString>>, CliDiagnostic> {
    if since.is_some() {
        if !changed {
            return Err(CliDiagnostic::incompatible_arguments("since", "changed"));
        }
        if staged {
            return Err(CliDiagnostic::incompatible_arguments("since", "staged"));
        }
    }

    if changed {
        if staged {
            return Err(CliDiagnostic::incompatible_arguments("changed", "staged"));
        }
        Ok(Some(get_changed_files(fs, configuration, since)?))
    } else if staged {
        Ok(Some(get_staged_files(fs)?))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests that all CLI options adhere to the invariants expected by `bpaf`.
    #[test]
    fn check_options() {
        pgt_command().check_invariants(false);
    }
}
