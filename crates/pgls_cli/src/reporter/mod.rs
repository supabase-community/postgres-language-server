pub(crate) mod github;
pub(crate) mod gitlab;
pub(crate) mod json;
pub(crate) mod junit;
pub(crate) mod summary;
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

    /// Whether a reporter prints the diagnostic: it must reach `--diagnostic-level`, and
    /// diagnostics tagged as verbose only print with `--verbose`.
    pub fn should_emit(&self, diagnostic: &Error) -> bool {
        diagnostic.severity() >= self.diagnostic_level
            && (self.verbose || !diagnostic.tags().is_verbose())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportMode {
    Terminal,
    GitHub,
    GitLab,
    Junit,
    Json,
    JsonPretty,
    Summary,
}

impl From<CliReporter> for ReportMode {
    fn from(value: CliReporter) -> Self {
        match value {
            CliReporter::Default => Self::Terminal,
            CliReporter::GitHub => Self::GitHub,
            CliReporter::Junit => Self::Junit,
            CliReporter::GitLab => Self::GitLab,
            CliReporter::Json => Self::Json,
            CliReporter::JsonPretty => Self::JsonPretty,
            CliReporter::Summary => Self::Summary,
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
    /// The unsafe fixes that `--fix` didn't apply.
    pub suggested_fixes_skipped: u32,
    /// The diagnostics with a fix, among those reported.
    pub fixable_diagnostics: u32,
    /// Whether the command writes files, e.g. with `--fix`.
    pub writes: bool,
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

    /// Adds a diagnostic that does not belong to a single file, ahead of the file diagnostics.
    pub fn prepend_diagnostic(&mut self, diagnostic: Error) {
        let (errors, warnings) = count_levels(std::slice::from_ref(&diagnostic));
        self.errors += errors;
        self.warnings += warnings;
        self.diagnostics.insert(0, diagnostic);
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
            ReportMode::Json => Box::new(json::JsonReportWriter { pretty: false }),
            ReportMode::JsonPretty => Box::new(json::JsonReportWriter { pretty: true }),
            ReportMode::Summary => Box::new(summary::SummaryReportWriter),
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

#[cfg(test)]
mod tests {
    use super::{Report, ReportConfig, ReportMode, Reporter};
    use pgls_console::{BufferConsole, StdDisplay};
    use pgls_diagnostics::{Diagnostic, DiagnosticExt, DiagnosticTags, Error, Severity};
    use std::time::Duration;

    #[derive(Debug, Diagnostic)]
    #[diagnostic(category = "syntax", severity = Error, message = "Invalid statement")]
    struct TestDiagnostic;

    fn diagnostic(path: &str) -> Error {
        TestDiagnostic
            .with_file_path(path)
            .with_file_span(0u32..6)
            .with_file_source_code("select;")
    }

    fn report_output(mode: ReportMode, verbose: bool) -> String {
        let report = Report::new(
            vec![
                diagnostic("regular.sql"),
                diagnostic("verbose.sql").with_tags(DiagnosticTags::VERBOSE),
            ],
            Duration::ZERO,
            0,
            None,
        );
        let mut reporter = Reporter::new(ReportConfig {
            mode,
            verbose,
            diagnostic_level: Severity::Hint,
            error_on_warnings: false,
            no_errors_on_unmatched: false,
        });
        let mut console = BufferConsole::default();
        reporter.report(&mut console, "check", &report).unwrap();

        console
            .out_buffer
            .iter()
            .map(|message| StdDisplay(&message.content).to_string())
            .collect()
    }

    #[test]
    fn every_reporter_prints_verbose_diagnostics_only_with_verbose() {
        for mode in [
            ReportMode::Terminal,
            ReportMode::GitHub,
            ReportMode::GitLab,
            ReportMode::Junit,
            ReportMode::Json,
            ReportMode::JsonPretty,
            ReportMode::Summary,
        ] {
            for verbose in [false, true] {
                let output = report_output(mode, verbose);
                assert!(
                    output.contains("regular.sql"),
                    "{mode:?} (verbose: {verbose}) drops the regular diagnostic:\n{output}"
                );
                assert_eq!(
                    output.contains("verbose.sql"),
                    verbose,
                    "{mode:?} (verbose: {verbose}) gets the verbose diagnostic wrong:\n{output}"
                );
            }
        }
    }
}
