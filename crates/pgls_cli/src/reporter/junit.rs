use crate::diagnostics::CliDiagnostic;
use crate::reporter::{Report, ReportConfig, ReportWriter};
use pgls_console::{Console, ConsoleExt, markup};
use pgls_diagnostics::display::SourceFile;
use pgls_diagnostics::{Error, Resource};
use quick_junit::{NonSuccessKind, Report as JunitReport, TestCase, TestCaseStatus, TestSuite};
use std::fmt::{Display, Formatter};

pub(crate) struct JunitReportWriter;

impl ReportWriter for JunitReportWriter {
    fn write(
        &mut self,
        console: &mut dyn Console,
        command_name: &str,
        report: &Report,
        config: &ReportConfig,
    ) -> Result<(), CliDiagnostic> {
        let mut junit = JunitReport::new(command_name);
        junit.time = Some(report.duration);
        junit.errors = report.errors as usize;
        append_diagnostics(&mut junit, config, &report.diagnostics)?;

        console.log(markup! {{junit.to_string().unwrap()}});
        Ok(())
    }
}

fn append_diagnostics(
    report: &mut JunitReport,
    config: &ReportConfig,
    diagnostics: &[Error],
) -> Result<(), CliDiagnostic> {
    for diagnostic in diagnostics.iter().filter(|diag| should_emit(config, diag)) {
        let mut status = TestCaseStatus::non_success(NonSuccessKind::Failure);
        let message = format!("{}", JunitDiagnostic { diagnostic });
        status.set_message(message.clone());

        let location = diagnostic.location();

        if let (Some(span), Some(source_code), Some(resource)) =
            (location.span, location.source_code, location.resource)
        {
            let source = SourceFile::new(source_code);
            let start = source
                .location(span.start())
                .map_err(CliDiagnostic::io_error)?;

            status.set_description(format!(
                "line {row:?}, col {col:?}, {body}",
                row = start.line_number.to_zero_indexed(),
                col = start.column_number.to_zero_indexed(),
                body = message
            ));
            let mut case = TestCase::new(
                format!(
                    "org.pgls.{}",
                    diagnostic
                        .category()
                        .map(|c| c.name())
                        .unwrap_or_default()
                        .replace('/', ".")
                ),
                status,
            );

            if let Resource::File(path) = resource {
                let mut test_suite = TestSuite::new(path);
                case.extra
                    .insert("line".into(), start.line_number.get().to_string().into());
                case.extra.insert(
                    "column".into(),
                    start.column_number.get().to_string().into(),
                );
                test_suite.extra.insert("package".into(), "org.pgls".into());
                test_suite.add_test_case(case);
                report.add_test_suite(test_suite);
            }
        }
    }

    Ok(())
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

struct JunitDiagnostic<'a> {
    diagnostic: &'a Error,
}

impl Display for JunitDiagnostic<'_> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        self.diagnostic.description(fmt)
    }
}
