use crate::diagnostics::CliDiagnostic;
use crate::reporter::{Report, ReportConfig, ReportWriter};
use pgls_console::{Console, ConsoleExt, markup};
use pgls_diagnostics::display::SourceFile;
use pgls_diagnostics::{Error, PrintDescription, Resource, Severity};
use serde::Serialize;

pub(crate) struct JsonReportWriter {
    pub pretty: bool,
}

impl ReportWriter for JsonReportWriter {
    fn write(
        &mut self,
        console: &mut dyn Console,
        command_name: &str,
        report: &Report,
        config: &ReportConfig,
    ) -> Result<(), CliDiagnostic> {
        let diagnostics: Vec<_> = report
            .diagnostics
            .iter()
            .filter(|d| d.severity() >= config.diagnostic_level)
            .filter(|d| {
                if d.tags().is_verbose() {
                    config.verbose
                } else {
                    true
                }
            })
            .map(to_json_report)
            .collect();

        let summary = JsonSummary::from_report(report);

        let output = JsonOutput {
            summary,
            diagnostics,
            command: command_name.to_string(),
        };

        let serialized = if self.pretty {
            serde_json::to_string_pretty(&output)
        } else {
            serde_json::to_string(&output)
        }
        .map_err(|e| CliDiagnostic::io_error(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        console.log(markup!({ serialized }));
        Ok(())
    }
}

#[derive(Serialize)]
struct JsonOutput {
    summary: JsonSummary,
    diagnostics: Vec<JsonDiagnostic>,
    command: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsonSummary {
    changed: usize,
    unchanged: usize,
    duration: u128,
    errors: u32,
    warnings: u32,
    skipped: usize,
    skipped_diagnostics: u32,
}

impl JsonSummary {
    fn from_report(report: &Report) -> Self {
        let (changed, unchanged, skipped) = report
            .traversal
            .as_ref()
            .map(|t| (t.changed, t.unchanged, t.skipped))
            .unwrap_or_default();
        Self {
            changed,
            unchanged,
            duration: report.duration.as_nanos(),
            errors: report.errors,
            warnings: report.warnings,
            skipped,
            skipped_diagnostics: report.skipped_diagnostics,
        }
    }
}

#[derive(Serialize)]
struct JsonDiagnostic {
    severity: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    location: Option<JsonLocation>,
}

#[derive(Serialize)]
struct JsonLocation {
    path: String,
    start: JsonPosition,
    end: JsonPosition,
}

#[derive(Serialize)]
struct JsonPosition {
    line: usize,
    column: usize,
}

fn to_json_report(diagnostic: &Error) -> JsonDiagnostic {
    let message = PrintDescription(diagnostic).to_string();
    let category = diagnostic.category().map(|c| c.name().to_string());
    let severity = match diagnostic.severity() {
        Severity::Hint => "hint",
        Severity::Information => "info",
        Severity::Warning => "warning",
        Severity::Error => "error",
        Severity::Fatal => "fatal",
    };

    let location = to_location(diagnostic);

    JsonDiagnostic {
        severity,
        message,
        category,
        location,
    }
}

fn to_location(diagnostic: &Error) -> Option<JsonLocation> {
    let loc = diagnostic.location();
    let path = match loc.resource {
        Some(Resource::File(file)) => file.to_string(),
        _ => return None,
    };

    match (loc.span, loc.source_code) {
        (Some(span), Some(source_code)) => {
            let source = SourceFile::new(source_code);
            let start = source.location(span.start()).ok()?;
            let end = source.location(span.end()).ok()?;
            Some(JsonLocation {
                path,
                start: JsonPosition {
                    line: start.line_number.get(),
                    column: start.column_number.get(),
                },
                end: JsonPosition {
                    line: end.line_number.get(),
                    column: end.column_number.get(),
                },
            })
        }
        _ => Some(JsonLocation {
            path,
            start: JsonPosition { line: 0, column: 0 },
            end: JsonPosition { line: 0, column: 0 },
        }),
    }
}
