use crate::diagnostics::CliDiagnostic;
use crate::reporter::{Report, ReportConfig, ReportWriter, TraversalData};
use pgls_console::fmt::Formatter;
use pgls_console::{Console, ConsoleExt, fmt, markup};
use pgls_diagnostics::advice::ListAdvice;
use pgls_diagnostics::{Diagnostic, Error, PrintDiagnostic};
use pgls_fs::PgLSPath;
use std::borrow::Cow;
use std::collections::BTreeSet;

pub(crate) struct TerminalReportWriter;

impl ReportWriter for TerminalReportWriter {
    fn write(
        &mut self,
        console: &mut dyn Console,
        command_name: &str,
        report: &Report,
        config: &ReportConfig,
    ) -> Result<(), CliDiagnostic> {
        log_diagnostics(console, config, &report.diagnostics);

        if let Some(traversal) = &report.traversal {
            console.log(markup! {
                {ConsoleTraversalSummary(command_name, report, traversal)}
            });
            if config.verbose {
                log_evaluated_paths(console, &traversal.evaluated_paths);
            }
        } else {
            console.log(markup! {
                {ConsoleDiagnosticSummary(command_name, report)}
            });
        }

        Ok(())
    }
}

fn log_diagnostics(console: &mut dyn Console, config: &ReportConfig, diagnostics: &[Error]) {
    for diagnostic in diagnostics {
        if diagnostic.severity() < config.diagnostic_level {
            continue;
        }

        if diagnostic.tags().is_verbose() && config.verbose {
            console.error(markup! {{PrintDiagnostic::verbose(diagnostic)}});
        } else if !diagnostic.tags().is_verbose() {
            console.error(markup! {{PrintDiagnostic::simple(diagnostic)}});
        }
    }
}

fn log_evaluated_paths(console: &mut dyn Console, evaluated_paths: &BTreeSet<PgLSPath>) {
    let evaluated_paths_diagnostic = EvaluatedPathsDiagnostic {
        advice: ListAdvice {
            list: evaluated_paths
                .iter()
                .map(|p| p.display().to_string())
                .collect(),
        },
    };

    let fixed_paths_diagnostic = FixedPathsDiagnostic {
        advice: ListAdvice {
            list: evaluated_paths
                .iter()
                .filter(|p| p.was_written())
                .map(|p| p.display().to_string())
                .collect(),
        },
    };

    console.log(markup! {
        {PrintDiagnostic::verbose(&evaluated_paths_diagnostic)}
    });
    console.log(markup! {
        {PrintDiagnostic::verbose(&fixed_paths_diagnostic)}
    });
}

#[derive(Debug, Diagnostic)]
#[diagnostic(
    tags(VERBOSE),
    severity = Information,
    message = "Files processed:"
)]
struct EvaluatedPathsDiagnostic {
    #[advice]
    advice: ListAdvice<String>,
}

#[derive(Debug, Diagnostic)]
#[diagnostic(
    tags(VERBOSE),
    severity = Information,
    message = "Files fixed:"
)]
struct FixedPathsDiagnostic {
    #[advice]
    advice: ListAdvice<String>,
}

struct Files(usize);

impl fmt::Display for Files {
    fn fmt(&self, fmt: &mut Formatter) -> std::io::Result<()> {
        fmt.write_markup(markup!({self.0} " "))?;
        if self.0 == 1 {
            fmt.write_str("file")
        } else {
            fmt.write_str("files")
        }
    }
}

struct SummaryDetail(usize);

impl fmt::Display for SummaryDetail {
    fn fmt(&self, fmt: &mut Formatter) -> std::io::Result<()> {
        if self.0 > 0 {
            fmt.write_markup(markup! {
                " Fixed "{Files(self.0)}"."
            })
        } else {
            fmt.write_markup(markup! {
                " No fixes applied."
            })
        }
    }
}

struct ConsoleTraversalSummary<'a>(&'a str, &'a Report, &'a TraversalData);

impl fmt::Display for ConsoleTraversalSummary<'_> {
    fn fmt(&self, fmt: &mut Formatter) -> std::io::Result<()> {
        let action = traversal_action(self.0);
        let files = Files(self.2.changed + self.2.unchanged);
        fmt.write_markup(markup!(
            <Info>
                {action} " "{files}" in "{self.1.duration}"." {SummaryDetail(self.2.changed)}
            </Info>
        ))?;

        if self.1.errors > 0 {
            if self.1.errors == 1 {
                fmt.write_markup(markup!(
                    "\n"<Error>"Found "{self.1.errors}" error."</Error>
                ))?;
            } else {
                fmt.write_markup(markup!(
                    "\n"<Error>"Found "{self.1.errors}" errors."</Error>
                ))?;
            }
        }
        if self.1.warnings > 0 {
            if self.1.warnings == 1 {
                fmt.write_markup(markup!(
                    "\n"<Warn>"Found "{self.1.warnings}" warning."</Warn>
                ))?;
            } else {
                fmt.write_markup(markup!(
                    "\n"<Warn>"Found "{self.1.warnings}" warnings."</Warn>
                ))?;
            }
        }
        Ok(())
    }
}

struct ConsoleDiagnosticSummary<'a>(&'a str, &'a Report);

impl fmt::Display for ConsoleDiagnosticSummary<'_> {
    fn fmt(&self, fmt: &mut Formatter) -> std::io::Result<()> {
        let action = diagnostic_action(self.0);
        fmt.write_markup(markup! {
            <Info>
                {action} " completed in "{self.1.duration}"."
            </Info>
        })?;

        if self.1.errors > 0 {
            fmt.write_markup(markup!(
                "\n"<Error>"Found "{self.1.errors}" error(s)."</Error>
            ))?;
        }

        if self.1.warnings > 0 {
            fmt.write_markup(markup!(
                "\n"<Warn>"Found "{self.1.warnings}" warning(s)."</Warn>
            ))?;
        }
        Ok(())
    }
}

fn traversal_action(command: &str) -> Cow<'static, str> {
    match command {
        "check" => Cow::Borrowed("Checked"),
        _ => Cow::Borrowed("Processed"),
    }
}

fn diagnostic_action(command: &str) -> Cow<'static, str> {
    match command {
        "check" => Cow::Borrowed("Check"),
        _ => Cow::Borrowed("Command"),
    }
}
