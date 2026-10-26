use crate::diagnostics::CliDiagnostic;
use crate::reporter::{Report, ReportConfig, ReportWriter};
use pgls_console::{Console, ConsoleExt, markup};
use pgls_diagnostics::PrintGitHubDiagnostic;

pub(crate) struct GithubReportWriter;

impl ReportWriter for GithubReportWriter {
    fn write(
        &mut self,
        console: &mut dyn Console,
        _command_name: &str,
        report: &Report,
        config: &ReportConfig,
    ) -> Result<(), CliDiagnostic> {
        for diagnostic in &report.diagnostics {
            if config.should_emit(diagnostic) {
                console.log(markup! {{PrintGitHubDiagnostic(diagnostic)}});
            }
        }

        Ok(())
    }
}
