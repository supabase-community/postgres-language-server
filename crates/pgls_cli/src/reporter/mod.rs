pub(crate) mod github;
pub(crate) mod gitlab;
pub(crate) mod junit;
pub(crate) mod terminal;

use crate::cli_options::{CliOptions, CliReporter};
use crate::diagnostics::CliDiagnostic;
use pgls_console::Console;
use pgls_diagnostics::{Error, Severity};
use pgls_fs::PgLSPath;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ReportConfig {
    pub mode: ReportMode,
    pub verbose: bool,
    pub diagnostic_level: Severity,
    pub error_on_warnings: bool,
    pub no_errors_on_unmatched: bool,
}

impl ReportConfig {
    pub fn from_cli_options(cli_options: &CliOptions) -> Self {
        Self {
            mode: cli_options.reporter.clone().into(),
            verbose: cli_options.verbose,
            diagnostic_level: cli_options.diagnostic_level,
            error_on_warnings: cli_options.error_on_warnings,
            no_errors_on_unmatched: cli_options.no_errors_on_unmatched,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportMode {
    Terminal,
    GitHub,
    GitLab,
    Junit,
}

impl From<CliReporter> for ReportMode {
    fn from(value: CliReporter) -> Self {
        match value {
            CliReporter::Default => Self::Terminal,
            CliReporter::GitHub => Self::GitHub,
            CliReporter::Junit => Self::Junit,
            CliReporter::GitLab => Self::GitLab,
        }
    }
}

#[derive(Debug)]
pub struct TraversalData {
    pub evaluated_paths: BTreeSet<PgLSPath>,
    pub changed: usize,
    pub unchanged: usize,
    pub matches: usize,
    pub skipped: usize,
    pub suggested_fixes_skipped: u32,
    pub diagnostics_not_printed: u32,
    pub workspace_root: Option<PathBuf>,
}

#[derive(Debug)]
pub struct Report {
    pub diagnostics: Vec<Error>,
    pub duration: Duration,
    pub errors: u32,
    pub warnings: u32,
    pub skipped_diagnostics: u32,
    pub traversal: Option<TraversalData>,
}

impl Report {
    pub fn new(
        diagnostics: Vec<Error>,
        duration: Duration,
        skipped_diagnostics: u32,
        traversal: Option<TraversalData>,
    ) -> Self {
        let (errors, warnings) = count_levels(&diagnostics);
        Self {
            diagnostics,
            duration,
            errors,
            warnings,
            skipped_diagnostics,
            traversal,
        }
    }
}

pub trait ReportWriter {
    fn write(
        &mut self,
        console: &mut dyn Console,
        command_name: &str,
        payload: &Report,
        config: &ReportConfig,
    ) -> Result<(), CliDiagnostic>;
}

pub struct Reporter {
    config: ReportConfig,
}

impl Reporter {
    pub fn from_cli_options(cli_options: &CliOptions) -> Self {
        Self {
            config: ReportConfig::from_cli_options(cli_options),
        }
    }

    pub fn new(config: ReportConfig) -> Self {
        Self { config }
    }

    pub fn report(
        &mut self,
        console: &mut dyn Console,
        command_name: &str,
        payload: &Report,
    ) -> Result<(), CliDiagnostic> {
        let mut writer: Box<dyn ReportWriter> = match self.config.mode {
            ReportMode::Terminal => Box::new(terminal::TerminalReportWriter),
            ReportMode::GitHub => Box::new(github::GithubReportWriter),
            ReportMode::GitLab => Box::new(gitlab::GitLabReportWriter),
            ReportMode::Junit => Box::new(junit::JunitReportWriter),
        };

        writer.write(console, command_name, payload, &self.config)
    }
}

fn count_levels(diagnostics: &[Error]) -> (u32, u32) {
    let mut errors = 0u32;
    let mut warnings = 0u32;
    for diag in diagnostics {
        match diag.severity() {
            Severity::Error | Severity::Fatal => errors += 1,
            Severity::Warning => warnings += 1,
            _ => {}
        }
    }
    (errors, warnings)
}
