use crate::changed::{get_changed_files, get_staged_files};
use crate::cli_options::{CliOptions, CliReporter, ColorsArg, cli_options};
use crate::logging::LoggingKind;
use crate::{CliDiagnostic, LoggingLevel, VERSION};
use bpaf::Bpaf;
use pgls_configuration::{PartialConfiguration, partial_configuration};
use pgls_fs::FileSystem;
use pgls_workspace::DynRef;
use std::ffi::OsString;
use std::path::PathBuf;
pub(crate) mod check;
pub(crate) mod clean;
pub(crate) mod daemon;
pub(crate) mod dblint;
pub(crate) mod init;
pub(crate) mod version;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options, version(VERSION))]
#[allow(clippy::large_enum_variant)]
/// Postgres Tools official CLI. Use it to check the health of your project or run it to check single files.
pub enum PgLSCommand {
    /// Shows the version information and quit.
    #[bpaf(command)]
    Version(#[bpaf(external(cli_options), hide_usage)] CliOptions),

    /// Runs everything to the requested files.
    #[bpaf(command)]
    Dblint {
        #[bpaf(external(partial_configuration), hide_usage, optional)]
        configuration: Option<PartialConfiguration>,

        #[bpaf(external, hide_usage)]
        cli_options: CliOptions,
    },

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

impl PgLSCommand {
    const fn cli_options(&self) -> Option<&CliOptions> {
        match self {
            PgLSCommand::Version(cli_options)
            | PgLSCommand::Check { cli_options, .. }
            | PgLSCommand::Dblint { cli_options, .. } => Some(cli_options),
            PgLSCommand::LspProxy { .. }
            | PgLSCommand::Start { .. }
            | PgLSCommand::Stop
            | PgLSCommand::Init
            | PgLSCommand::RunServer { .. }
            | PgLSCommand::Clean
            | PgLSCommand::PrintSocket => None,
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

pub(crate) fn get_files_to_process_with_cli_options(
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
        pg_l_s_command().check_invariants(false);
    }
}
