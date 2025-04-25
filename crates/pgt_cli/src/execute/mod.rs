mod diagnostics;
mod process_file;
mod std_in;
pub(crate) mod traverse;

use crate::cli_options::{CliOptions, CliReporter};
use crate::execute::traverse::{TraverseResult, traverse};
use crate::reporter::github::{GithubReporter, GithubReporterVisitor};
use crate::reporter::gitlab::{GitLabReporter, GitLabReporterVisitor};
use crate::reporter::junit::{JunitReporter, JunitReporterVisitor};
use crate::reporter::terminal::{ConsoleReporter, ConsoleReporterVisitor};
use crate::{CliDiagnostic, CliSession, DiagnosticsPayload, Reporter};
use pgt_diagnostics::{Category, category};
use std::borrow::Borrow;
use std::ffi::OsString;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use tracing::info;

/// Useful information during the traversal of files and virtual content
#[derive(Debug, Clone)]
pub struct Execution {
    /// How the information should be collected and reported
    report_mode: ReportMode,

    /// The modality of execution of the traversal
    traversal_mode: TraversalMode,

    /// The maximum number of diagnostics that can be printed in console
    max_diagnostics: u32,
}

impl Execution {
    pub fn report_mode(&self) -> &ReportMode {
        &self.report_mode
    }
}

/// A type that holds the information to execute the CLI via `stdin
#[derive(Debug, Clone)]
pub struct Stdin(
    #[allow(unused)]
    /// The virtual path to the file
    PathBuf,
    /// The content of the file
    String,
);

impl Stdin {
    fn as_content(&self) -> &str {
        self.1.as_str()
    }
}

impl From<(PathBuf, String)> for Stdin {
    fn from((path, content): (PathBuf, String)) -> Self {
        Self(path, content)
    }
}

#[derive(Debug, Clone)]
pub struct VcsTargeted {
    pub staged: bool,
    pub changed: bool,
}

impl From<(bool, bool)> for VcsTargeted {
    fn from((staged, changed): (bool, bool)) -> Self {
        Self { staged, changed }
    }
}

#[derive(Debug, Clone)]
pub enum TraversalMode {
    /// A dummy mode to be used when the CLI is not running any command
    Dummy,
    /// This mode is enabled when running the command `check`
    Check {
        // The type of fixes that should be applied when analyzing a file.
        //
        // It's [None] if the `check` command is called without `--apply` or `--apply-suggested`
        // arguments.
        // fix_file_mode: Option<FixFileMode>,

        /// An optional tuple.
        /// 1. The virtual path to the file
        /// 2. The content of the file
        stdin: Option<Stdin>,
        /// A flag to know vcs integrated options such as `--staged` or `--changed` are enabled
        vcs_targeted: VcsTargeted,
    },
}

impl Display for TraversalMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TraversalMode::Dummy => write!(f, "dummy"),
            TraversalMode::Check { .. } => write!(f, "check"),
        }
    }
}

/// Tells to the execution of the traversal how the information should be reported
#[derive(Copy, Clone, Debug)]
pub enum ReportMode {
    /// Reports information straight to the console, it's the default mode
    Terminal,
    /// Reports information for GitHub
    GitHub,
    /// JUnit output
    /// Ref: https://github.com/testmoapp/junitxml?tab=readme-ov-file#basic-junit-xml-structure
    Junit,
    /// Reports information in the [GitLab Code Quality](https://docs.gitlab.com/ee/ci/testing/code_quality.html#implement-a-custom-tool) format.
    GitLab,
}

impl Default for ReportMode {
    fn default() -> Self {
        Self::Terminal {}
    }
}

impl From<CliReporter> for ReportMode {
    fn from(value: CliReporter) -> Self {
        match value {
            CliReporter::Default => Self::Terminal,
            CliReporter::GitHub => Self::GitHub,
            CliReporter::Junit => Self::Junit,
            CliReporter::GitLab => Self::GitLab {},
        }
    }
}

impl Execution {
    pub(crate) fn new(mode: TraversalMode) -> Self {
        Self {
            report_mode: ReportMode::default(),
            traversal_mode: mode,
            max_diagnostics: 20,
        }
    }

    /// It sets the reporting mode by reading the [CliOptions]
    pub(crate) fn set_report(mut self, cli_options: &CliOptions) -> Self {
        self.report_mode = cli_options.reporter.clone().into();
        self
    }

    pub(crate) fn traversal_mode(&self) -> &TraversalMode {
        &self.traversal_mode
    }

    pub(crate) fn get_max_diagnostics(&self) -> u32 {
        self.max_diagnostics
    }

    pub(crate) fn as_diagnostic_category(&self) -> &'static Category {
        match self.traversal_mode {
            TraversalMode::Dummy => category!("dummy"),
            TraversalMode::Check { .. } => category!("check"),
        }
    }

    /// Whether the traversal mode requires write access to files
    pub(crate) const fn requires_write_access(&self) -> bool {
        match self.traversal_mode {
            TraversalMode::Dummy => false,
            TraversalMode::Check { .. } => false,
        }
    }

    pub(crate) fn as_stdin_file(&self) -> Option<&Stdin> {
        match &self.traversal_mode {
            TraversalMode::Dummy => None,
            TraversalMode::Check { stdin, .. } => stdin.as_ref(),
        }
    }

    pub(crate) fn is_vcs_targeted(&self) -> bool {
        match &self.traversal_mode {
            TraversalMode::Dummy => false,
            TraversalMode::Check { vcs_targeted, .. } => {
                vcs_targeted.staged || vcs_targeted.changed
            }
        }
    }

    pub(crate) const fn is_check_apply(&self) -> bool {
        false
    }

    #[allow(unused)]
    /// Returns [true] if the user used the `--write`/`--fix` option
    pub(crate) fn is_write(&self) -> bool {
        match self.traversal_mode {
            TraversalMode::Dummy => false,
            TraversalMode::Check { .. } => false,
        }
    }
}

/// Based on the [mode](TraversalMode), the function might launch a traversal of the file system
/// or handles the stdin file.
pub fn execute_mode(
    mut execution: Execution,
    mut session: CliSession,
    cli_options: &CliOptions,
    paths: Vec<OsString>,
) -> Result<(), CliDiagnostic> {
    // If a custom reporter was provided, let's lift the limit so users can see all of them
    execution.max_diagnostics = if cli_options.reporter.is_default() {
        cli_options.max_diagnostics.into()
    } else {
        info!(
            "Removing the limit of --max-diagnostics, because of a reporter different from the default one: {}",
            cli_options.reporter
        );
        u32::MAX
    };

    // don't do any traversal if there's some content coming from stdin
    if let Some(stdin) = execution.as_stdin_file() {
        std_in::run(session, stdin.as_content())
    } else {
        let TraverseResult {
            summary,
            evaluated_paths,
            diagnostics,
        } = traverse(&execution, &mut session, cli_options, paths)?;
        let console = session.app.console;
        let errors = summary.errors;
        let skipped = summary.skipped;
        let processed = summary.changed + summary.unchanged;
        let should_exit_on_warnings = summary.warnings > 0 && cli_options.error_on_warnings;

        match execution.report_mode {
            ReportMode::Terminal => {
                let reporter = ConsoleReporter {
                    summary,
                    diagnostics_payload: DiagnosticsPayload {
                        verbose: cli_options.verbose,
                        diagnostic_level: cli_options.diagnostic_level,
                        diagnostics,
                    },
                    execution: execution.clone(),
                    evaluated_paths,
                };
                reporter.write(&mut ConsoleReporterVisitor(console))?;
            }
            ReportMode::GitHub => {
                let reporter = GithubReporter {
                    diagnostics_payload: DiagnosticsPayload {
                        verbose: cli_options.verbose,
                        diagnostic_level: cli_options.diagnostic_level,
                        diagnostics,
                    },
                    execution: execution.clone(),
                };
                reporter.write(&mut GithubReporterVisitor(console))?;
            }
            ReportMode::GitLab => {
                let reporter = GitLabReporter {
                    diagnostics: DiagnosticsPayload {
                        verbose: cli_options.verbose,
                        diagnostic_level: cli_options.diagnostic_level,
                        diagnostics,
                    },
                    execution: execution.clone(),
                };
                reporter.write(&mut GitLabReporterVisitor::new(
                    console,
                    session.app.fs.borrow().working_directory(),
                ))?;
            }
            ReportMode::Junit => {
                let reporter = JunitReporter {
                    summary,
                    diagnostics_payload: DiagnosticsPayload {
                        verbose: cli_options.verbose,
                        diagnostic_level: cli_options.diagnostic_level,
                        diagnostics,
                    },
                    execution: execution.clone(),
                };
                reporter.write(&mut JunitReporterVisitor::new(console))?;
            }
        }

        // Processing emitted error diagnostics, exit with a non-zero code
        if processed.saturating_sub(skipped) == 0 && !cli_options.no_errors_on_unmatched {
            Err(CliDiagnostic::no_files_processed())
        } else if errors > 0 || should_exit_on_warnings {
            let category = execution.as_diagnostic_category();
            if should_exit_on_warnings {
                if execution.is_check_apply() {
                    Err(CliDiagnostic::apply_warnings(category))
                } else {
                    Err(CliDiagnostic::check_warnings(category))
                }
            } else if execution.is_check_apply() {
                Err(CliDiagnostic::apply_error(category))
            } else {
                Err(CliDiagnostic::check_error(category))
            }
        } else {
            Ok(())
        }
    }
}
