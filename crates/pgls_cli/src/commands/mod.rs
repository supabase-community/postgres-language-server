use crate::changed::{get_changed_files, get_staged_files};
use crate::cli_options::{CliOptions, CliReporter, ColorsArg, cli_options};
use crate::execute::{StdinPayload, VcsTargeting};
use crate::logging::LoggingKind;
use crate::reporter::Report;
use crate::{CliDiagnostic, LoggingLevel, VERSION};
use bpaf::Bpaf;
use pgls_configuration::{PartialConfiguration, partial_configuration};
use pgls_console::{Console, ConsoleExt, markup};
use pgls_diagnostics::{Category, Error, PrintDiagnostic};
use pgls_fs::FileSystem;
use pgls_workspace::DynRef;
use std::ffi::OsString;
use std::path::PathBuf;
pub(crate) mod check;
pub(crate) mod clean;
pub(crate) mod daemon;
pub(crate) mod dblint;
pub(crate) mod format;
pub(crate) mod init;
pub(crate) mod parse;
pub(crate) mod schema_export;
pub(crate) mod version;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options, version(VERSION))]
#[allow(clippy::large_enum_variant)]
/// Postgres Language Server official CLI. Use it to check the health of your project or run it to check single files.
pub enum PgLSCommand {
    /// Shows the version information and quit.
    #[bpaf(command)]
    Version(#[bpaf(external(cli_options), hide_usage)] CliOptions),

    /// Lints your database schema.
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

        /// Apply the safe fixes of the lint rules and write the files.
        #[bpaf(long("fix"), switch)]
        fix: bool,

        /// Also apply the unsafe fixes, which may change what a statement does. Implies `--fix`.
        #[bpaf(long("unsafe"), switch)]
        unsafe_: bool,

        /// Single file, single path or list of paths
        #[bpaf(positional("PATH"), many)]
        paths: Vec<OsString>,
    },

    /// Formats the requested files.
    #[bpaf(command)]
    Format {
        #[bpaf(external(partial_configuration), hide_usage, optional)]
        configuration: Option<PartialConfiguration>,

        #[bpaf(external, hide_usage)]
        cli_options: CliOptions,

        /// Write formatted output back to files
        #[bpaf(long("write"), switch)]
        write: bool,

        /// Use this option when you want to format code piped from `stdin`, and print the output to `stdout`.
        ///
        /// The file doesn't need to exist on disk. The path is used to resolve the configuration
        /// (e.g. ignored files) and to label diagnostics, which are printed to `stderr`.
        ///
        /// Example: `echo 'select 1' | postgres-language-server format --stdin-file-path=test.sql`
        #[bpaf(long("stdin-file-path"), argument("PATH"), hide_usage)]
        stdin_file_path: Option<String>,

        /// When set to true, only the files that have been staged (the ones prepared to be committed)
        /// will be formatted. This option should be used when working locally.
        #[bpaf(long("staged"), switch)]
        staged: bool,

        /// When set to true, only the files that have been changed compared to your `defaultBranch`
        /// configuration will be formatted. This option should be used in CI environments.
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
            env("PGLS_LOG_PREFIX_NAME"),
            env("PGT_LOG_PREFIX_NAME"),
            long("log-prefix-name"),
            argument("STRING"),
            hide_usage,
            fallback(String::from("server.log")),
            display_fallback
        )]
        log_prefix_name: String,

        /// Allows to change the folder where logs are stored.
        #[bpaf(
            env("PGLS_LOG_PATH"),
            env("PGT_LOG_PATH"),
            long("log-path"),
            argument("PATH"),
            hide_usage,
            fallback(pgls_fs::ensure_cache_dir().join("pgls-logs")),
        )]
        log_path: PathBuf,
        /// Allows to set a custom file path to the configuration file,
        /// or a custom directory path to find `postgres-language-server.jsonc`
        #[bpaf(
            env("PGLS_CONFIG_PATH"),
            env("PGT_CONFIG_PATH"),
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
            env("PGLS_LOG_PREFIX_NAME"),
            env("PGT_LOG_PREFIX_NAME"),
            long("log-prefix-name"),
            argument("STRING"),
            hide_usage,
            fallback(String::from("server.log")),
            display_fallback
        )]
        log_prefix_name: String,
        /// Allows to change the folder where logs are stored.
        #[bpaf(
            env("PGLS_LOG_PATH"),
            env("PGT_LOG_PATH"),
            long("log-path"),
            argument("PATH"),
            hide_usage,
            fallback(pgls_fs::ensure_cache_dir().join("pgls-logs")),
        )]
        log_path: PathBuf,
        /// Allows to set a custom file path to the configuration file,
        /// or a custom directory path to find `postgres-language-server.jsonc`
        #[bpaf(
            env("PGLS_CONFIG_PATH"),
            env("PGT_CONFIG_PATH"),
            long("config-path"),
            argument("PATH")
        )]
        config_path: Option<PathBuf>,
        /// Bogus argument to make the command work with vscode-languageclient
        #[bpaf(long("stdio"), hide, hide_usage, switch)]
        stdio: bool,
    },

    #[bpaf(command)]
    /// Parses a SQL file (or standard input) and writes the parse tree as a protobuf `ParseResult` message.
    #[bpaf(command)]
    Parse {
        /// Path to the SQL file to parse. When omitted, the SQL is read from standard input.
        #[bpaf(long("file"), argument("PATH"), optional)]
        file: Option<PathBuf>,

        /// Alias of --file, mirroring the format command.
        #[bpaf(long("stdin-file-path"), argument("PATH"), optional, hide_usage)]
        stdin_file_path: Option<String>,
    },

    /// Cleans the logs emitted by the daemon.
    Clean,

    /// Exports the database schema to JSON for use with WASM bindings.
    /// Writes to stdout by default, or to a file if --output is specified.
    #[bpaf(command("schema-export"))]
    SchemaExport {
        /// PostgreSQL connection string (e.g., postgres://user:pass@host/db).
        /// Can also be set via the `DATABASE_URL` environment variable.
        #[bpaf(long("connection-string"), short('c'), argument("URL"), optional)]
        connection_string: Option<String>,

        /// Output file path for the JSON schema (defaults to stdout if not specified)
        #[bpaf(long("output"), short('o'), argument("PATH"))]
        output: Option<PathBuf>,
    },

    #[bpaf(command("__run_server"), hide)]
    RunServer {
        /// Allows to change the prefix applied to the file name of the logs.
        #[bpaf(
            env("PGLS_LOG_PREFIX_NAME"),
            env("PGT_LOG_PREFIX_NAME"),
            long("log-prefix-name"),
            argument("STRING"),
            hide_usage,
            fallback(String::from("server.log")),
            display_fallback
        )]
        log_prefix_name: String,

        /// Allows to change the folder where logs are stored.
        #[bpaf(
            env("PGLS_LOG_PATH"),
            env("PGT_LOG_PATH"),
            long("log-path"),
            argument("PATH"),
            hide_usage,
            fallback(pgls_fs::ensure_cache_dir().join("pgls-logs")),
        )]
        log_path: PathBuf,

        /// Allows to change the log level. Default is debug. This will only affect "pgls*" crates. All others are logged with info level.
        #[bpaf(
            env("PGLS_LOG_LEVEL"),
            env("PGT_LOG_LEVEL"),
            long("log-level"),
            argument("trace|debug|info|warn|error|none"),
            fallback(String::from("debug"))
        )]
        log_level: String,

        /// Allows to change the logging format kind. Default is hierarchical.
        #[bpaf(
            env("PGLS_LOG_KIND"),
            env("PGT_LOG_KIND"),
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
            env("PGLS_CONFIG_PATH"),
            env("PGT_CONFIG_PATH"),
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
            | PgLSCommand::Dblint { cli_options, .. }
            | PgLSCommand::Format { cli_options, .. } => Some(cli_options),
            PgLSCommand::LspProxy { .. }
            | PgLSCommand::Start { .. }
            | PgLSCommand::Stop
            | PgLSCommand::Init
            | PgLSCommand::RunServer { .. }
            | PgLSCommand::Clean
            | PgLSCommand::SchemaExport { .. }
            | PgLSCommand::Parse { .. }
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

/// The files that `check` and `format` process: the given paths, the files changed or staged in
/// git, or the code piped to `stdin`.
pub(crate) struct FileTargets {
    pub paths: Vec<OsString>,
    pub stdin_file_path: Option<String>,
    pub staged: bool,
    pub changed: bool,
    pub since: Option<String>,
}

impl FileTargets {
    /// Rejects the arguments that can't be combined, before the workspace is set up.
    pub(crate) fn validate(&self) -> Result<(), CliDiagnostic> {
        if self.stdin_file_path.is_some() {
            let incompatible = [
                ("PATH", !self.paths.is_empty()),
                ("staged", self.staged),
                ("changed", self.changed),
            ];
            if let Some((argument, _)) = incompatible.iter().find(|(_, is_set)| *is_set) {
                return Err(CliDiagnostic::incompatible_arguments(
                    "stdin-file-path",
                    *argument,
                ));
            }
        }

        if self.since.is_some() {
            if !self.changed {
                return Err(CliDiagnostic::incompatible_arguments("since", "changed"));
            }
            if self.staged {
                return Err(CliDiagnostic::incompatible_arguments("since", "staged"));
            }
        }

        if self.changed && self.staged {
            return Err(CliDiagnostic::incompatible_arguments("changed", "staged"));
        }

        Ok(())
    }

    /// The paths to process: the changed or staged files, else the given paths, else the working
    /// directory. Empty for the code piped to `stdin`.
    pub(crate) fn resolve_paths(
        &self,
        fs: &DynRef<'_, dyn FileSystem>,
        configuration: &PartialConfiguration,
    ) -> Result<Vec<OsString>, CliDiagnostic> {
        if self.stdin_file_path.is_some() {
            return Ok(Vec::new());
        }

        let mut paths = if self.changed {
            get_changed_files(fs, configuration, self.since.as_deref())?
        } else if self.staged {
            get_staged_files(fs)?
        } else {
            self.paths.clone()
        };

        if paths.is_empty()
            && let Some(current_dir) = fs.working_directory()
        {
            paths.push(current_dir.into_os_string());
        }

        Ok(paths)
    }

    /// The paths whose nested configuration files apply: the virtual path of the code piped to
    /// `stdin`, or else the paths to process.
    pub(crate) fn nested_configuration_paths(&self, paths: &[OsString]) -> Vec<OsString> {
        match &self.stdin_file_path {
            Some(path) => vec![OsString::from(path)],
            None => paths.to_vec(),
        }
    }

    pub(crate) fn vcs(&self) -> VcsTargeting {
        VcsTargeting {
            staged: self.staged,
            changed: self.changed,
        }
    }
}

/// Fails the command if the report has errors, warnings with `--error-on-warnings`, or, for the
/// commands that process files, no processed file without `--no-errors-on-unmatched`.
pub(crate) fn enforce_exit_codes(
    cli_options: &CliOptions,
    report: &Report,
    category: &'static Category,
) -> Result<(), CliDiagnostic> {
    if let Some(traversal) = &report.traversal {
        let processed = traversal.changed + traversal.unchanged;
        if processed.saturating_sub(traversal.skipped) == 0 && !cli_options.no_errors_on_unmatched {
            return Err(CliDiagnostic::no_files_processed());
        }
    }

    if report.errors > 0 {
        return Err(CliDiagnostic::check_error(category));
    }

    if report.warnings > 0 && cli_options.error_on_warnings {
        return Err(CliDiagnostic::check_warnings(category));
    }

    Ok(())
}

/// Prints diagnostics to stderr, for the commands that write their result to stdout.
pub(crate) fn print_diagnostics(console: &mut dyn Console, diagnostics: Vec<Error>) {
    for diagnostic in diagnostics {
        console.error(markup! {{PrintDiagnostic::simple(&diagnostic)}});
    }
}

/// Reads the code piped to `stdin` for the given virtual file path.
pub(crate) fn read_stdin_payload(
    path: &str,
    console: &mut dyn Console,
    subcommand: &str,
) -> Result<StdinPayload, CliDiagnostic> {
    match console.read() {
        Some(content) => Ok(StdinPayload {
            path: path.into(),
            content,
        }),
        None => Err(CliDiagnostic::missing_argument("stdin", subcommand)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporter::TraversalData;
    use pgls_diagnostics::{Diagnostic, category};
    use std::collections::BTreeSet;
    use std::time::Duration;

    #[derive(Debug, Diagnostic)]
    #[diagnostic(category = "format", severity = Warning, message = "A warning")]
    struct TestWarning;

    fn cli_options(args: &[&str]) -> CliOptions {
        bpaf::Parser::to_options(super::cli_options())
            .run_inner(args)
            .unwrap()
    }

    fn file_targets() -> FileTargets {
        FileTargets {
            paths: Vec::new(),
            stdin_file_path: None,
            staged: false,
            changed: false,
            since: None,
        }
    }

    fn traversal(unchanged: usize) -> TraversalData {
        TraversalData {
            evaluated_paths: BTreeSet::new(),
            changed: 0,
            unchanged,
            matches: 0,
            skipped: 0,
            suggested_fixes_skipped: 0,
            diagnostics_not_printed: 0,
            workspace_root: None,
        }
    }

    #[test]
    fn stdin_file_path_rejects_the_other_targets() {
        let stdin = || FileTargets {
            stdin_file_path: Some("virtual.sql".into()),
            ..file_targets()
        };
        assert!(stdin().validate().is_ok());
        for targets in [
            FileTargets {
                paths: vec!["file.sql".into()],
                ..stdin()
            },
            FileTargets {
                staged: true,
                ..stdin()
            },
            FileTargets {
                changed: true,
                ..stdin()
            },
        ] {
            assert!(matches!(
                targets.validate(),
                Err(CliDiagnostic::IncompatibleArguments(_))
            ));
        }
    }

    #[test]
    fn warnings_fail_every_command_with_error_on_warnings() {
        let report = Report::new(
            vec![TestWarning.into()],
            Duration::ZERO,
            0,
            Some(traversal(1)),
        );
        let format = category!("format");

        assert!(enforce_exit_codes(&cli_options(&[]), &report, format).is_ok());
        assert!(matches!(
            enforce_exit_codes(&cli_options(&["--error-on-warnings"]), &report, format),
            Err(CliDiagnostic::CheckError(_))
        ));
    }

    #[test]
    fn only_commands_that_process_files_fail_without_files() {
        let without_traversal = Report::new(Vec::new(), Duration::ZERO, 0, None);
        let without_files = Report::new(Vec::new(), Duration::ZERO, 0, Some(traversal(0)));
        let check = category!("check");

        assert!(enforce_exit_codes(&cli_options(&[]), &without_traversal, check).is_ok());
        assert!(matches!(
            enforce_exit_codes(&cli_options(&[]), &without_files, check),
            Err(CliDiagnostic::NoFilesWereProcessed(_))
        ));
        assert!(
            enforce_exit_codes(
                &cli_options(&["--no-errors-on-unmatched"]),
                &without_files,
                check
            )
            .is_ok()
        );
    }

    /// Tests that all CLI options adhere to the invariants expected by `bpaf`.
    #[test]
    fn check_options() {
        pg_l_s_command().check_invariants(false);
    }
}
