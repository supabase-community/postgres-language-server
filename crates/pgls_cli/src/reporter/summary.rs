use crate::diagnostics::CliDiagnostic;
use crate::reporter::terminal::{ConsoleDiagnosticSummary, ConsoleTraversalSummary};
use crate::reporter::{Report, ReportConfig, ReportWriter};
use pgls_console::fmt::{Display, Formatter};
use pgls_console::{Console, ConsoleExt, markup};
use pgls_diagnostics::{Error, Resource, Severity};
use std::collections::BTreeMap;

pub(crate) struct SummaryReportWriter;

impl ReportWriter for SummaryReportWriter {
    fn write(
        &mut self,
        console: &mut dyn Console,
        command_name: &str,
        report: &Report,
        config: &ReportConfig,
    ) -> Result<(), CliDiagnostic> {
        let file_diagnostics = collect_file_diagnostics(report, config);
        if !file_diagnostics.0.is_empty() {
            console.log(markup! {{ file_diagnostics }});
        }

        if let Some(traversal) = &report.traversal {
            console.log(markup! {
                {ConsoleTraversalSummary(command_name, report, traversal)}
            });
        } else {
            console.log(markup! {
                {ConsoleDiagnosticSummary(command_name, report)}
            });
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
struct DiagnosticCounts {
    errors: usize,
    warnings: usize,
}

impl DiagnosticCounts {
    fn track(&mut self, severity: Severity) {
        match severity {
            Severity::Error | Severity::Fatal => self.errors += 1,
            Severity::Warning => self.warnings += 1,
            _ => {}
        }
    }
}

struct FileDiagnostics(BTreeMap<String, DiagnosticCounts>);

fn collect_file_diagnostics(report: &Report, config: &ReportConfig) -> FileDiagnostics {
    let mut files: BTreeMap<String, DiagnosticCounts> = BTreeMap::new();

    for diagnostic in &report.diagnostics {
        if !should_emit(config, diagnostic) {
            continue;
        }

        let path = match diagnostic.location().resource {
            Some(Resource::File(p)) => p.to_string(),
            _ => continue,
        };

        files.entry(path).or_default().track(diagnostic.severity());
    }

    FileDiagnostics(files)
}

fn should_emit(config: &ReportConfig, diagnostic: &Error) -> bool {
    if diagnostic.severity() < config.diagnostic_level {
        return false;
    }

    if diagnostic.tags().is_verbose() {
        config.verbose
    } else {
        true
    }
}

impl Display for FileDiagnostics {
    fn fmt(&self, fmt: &mut Formatter) -> std::io::Result<()> {
        for (path, counts) in &self.0 {
            fmt.write_str(path)?;
            fmt.write_str(": ")?;

            let mut parts = Vec::new();
            if counts.errors > 0 {
                parts.push(format!(
                    "{} {}",
                    counts.errors,
                    if counts.errors == 1 {
                        "error"
                    } else {
                        "errors"
                    }
                ));
            }
            if counts.warnings > 0 {
                parts.push(format!(
                    "{} {}",
                    counts.warnings,
                    if counts.warnings == 1 {
                        "warning"
                    } else {
                        "warnings"
                    }
                ));
            }
            fmt.write_str(&parts.join(", "))?;
            fmt.write_str("\n")?;
        }
        Ok(())
    }
}
