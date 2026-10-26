use crate::display::frame::SourceFile;
use crate::{Diagnostic, Resource, Severity, diagnostic::internal::AsDiagnostic};
use pgls_console::{MarkupBuf, fmt, markup};
use pgls_text_size::{TextRange, TextSize};
use std::io;

/// Helper struct for printing a diagnostic as markup into any formatter
/// implementing [pgls_console::fmt::Write].
pub struct PrintGitHubDiagnostic<'fmt, D: ?Sized>(pub &'fmt D);

impl<D: AsDiagnostic + ?Sized> fmt::Display for PrintGitHubDiagnostic<'_, D> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> io::Result<()> {
        let diagnostic = self.0.as_diagnostic();
        let location = diagnostic.location();

        // Docs:
        // https://docs.github.com/en/actions/using-workflows/workflow-commands-for-github-actions
        let command = match diagnostic.severity() {
            Severity::Error | Severity::Fatal => "error",
            Severity::Warning => "warning",
            Severity::Hint | Severity::Information => "notice",
        };

        let message = {
            let mut message = MarkupBuf::default();
            let mut fmt = fmt::Formatter::new(&mut message);
            fmt.write_markup(markup!({ PrintDiagnosticMessage(diagnostic) }))?;
            markup_to_string(&message)
        };
        let message = message.map_or_else(String::new, escape_data);

        let title = {
            diagnostic
                .category()
                .map(|category| category.name())
                .unwrap_or_default()
        };

        // Diagnostics without a file, like a failed database connection or a configuration
        // error, become an annotation of the workflow run instead of being dropped.
        let file_name_unescaped = match &location.resource {
            Some(Resource::File(file)) => file,
            _ => {
                return fmt.write_str(format!("::{command} title={title}::{message}").as_str());
            }
        };
        let file = escape_property(file_name_unescaped);

        // Without the source we can't compute a line, so the annotation applies to the file.
        let Some(source_code) = location.source_code else {
            return fmt
                .write_str(format!("::{command} title={title},file={file}::{message}").as_str());
        };

        let span = location
            .span
            // We fall back to 1:1. This usually covers diagnostics that belong to the formatter or organize imports
            .unwrap_or(TextRange::new(TextSize::from(1), TextSize::from(1)));

        let source = SourceFile::new(source_code);
        let start = source.location(span.start())?;
        let end = source.location(span.end())?;

        fmt.write_str(
            format!(
                "::{} title={},file={},line={},endLine={},col={},endColumn={}::{}",
                command, // constant, doesn't need escaping
                title,   // the diagnostic category
                file,
                start.line_number,   // integer, doesn't need escaping
                end.line_number,     // integer, doesn't need escaping
                start.column_number, // integer, doesn't need escaping
                end.column_number,   // integer, doesn't need escaping
                message,
            )
            .as_str(),
        )?;

        Ok(())
    }
}

struct PrintDiagnosticMessage<'fmt, D: ?Sized>(&'fmt D);

impl<D: Diagnostic + ?Sized> fmt::Display for PrintDiagnosticMessage<'_, D> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> io::Result<()> {
        let Self(diagnostic) = *self;
        diagnostic.message(fmt)?;
        Ok(())
    }
}

fn escape_data<S: AsRef<str>>(value: S) -> String {
    let value = value.as_ref();

    // Refs:
    // - https://github.com/actions/runner/blob/a4c57f27477077e57545af79851551ff7f5632bd/src/Runner.Common/ActionCommand.cs#L18-L22
    // - https://github.com/actions/toolkit/blob/fe3e7ce9a7f995d29d1fcfd226a32bca407f9dc8/packages/core/src/command.ts#L80-L94
    let mut result = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '\r' => result.push_str("%0D"),
            '\n' => result.push_str("%0A"),
            '%' => result.push_str("%25"),
            _ => result.push(c),
        }
    }
    result
}

fn escape_property<S: AsRef<str>>(value: S) -> String {
    let value = value.as_ref();

    // Refs:
    // - https://github.com/actions/runner/blob/a4c57f27477077e57545af79851551ff7f5632bd/src/Runner.Common/ActionCommand.cs#L25-L32
    // - https://github.com/actions/toolkit/blob/fe3e7ce9a7f995d29d1fcfd226a32bca407f9dc8/packages/core/src/command.ts#L80-L94
    let mut result = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '\r' => result.push_str("%0D"),
            '\n' => result.push_str("%0A"),
            ':' => result.push_str("%3A"),
            ',' => result.push_str("%2C"),
            '%' => result.push_str("%25"),
            _ => result.push(c),
        }
    }
    result
}

fn markup_to_string(markup: &MarkupBuf) -> Option<String> {
    let mut buffer = Vec::new();
    let mut write = fmt::Termcolor(termcolor::NoColor::new(&mut buffer));
    let mut fmt = fmt::Formatter::new(&mut write);
    fmt.write_markup(markup! { {markup} }).ok()?;
    String::from_utf8(buffer).ok()
}

#[cfg(test)]
mod tests {
    use std::io;

    use pgls_console::{fmt, markup};
    use pgls_diagnostics_categories::{Category, category};
    use pgls_text_size::{TextRange, TextSize};

    use super::{PrintGitHubDiagnostic, markup_to_string};
    use crate::{Diagnostic, Location, Severity};

    #[derive(Debug)]
    struct TestDiagnostic {
        path: Option<String>,
        span: Option<TextRange>,
        source_code: Option<String>,
    }

    impl Diagnostic for TestDiagnostic {
        fn category(&self) -> Option<&'static Category> {
            Some(category!("internalError/io"))
        }

        fn severity(&self) -> Severity {
            Severity::Error
        }

        fn message(&self, fmt: &mut fmt::Formatter<'_>) -> io::Result<()> {
            write!(fmt, "something failed")
        }

        fn location(&self) -> Location<'_> {
            Location::builder()
                .resource(&self.path)
                .span(&self.span)
                .source_code(&self.source_code)
                .build()
        }
    }

    fn print(diagnostic: &TestDiagnostic) -> String {
        let mut markup = pgls_console::MarkupBuf::default();
        fmt::Formatter::new(&mut markup)
            .write_markup(markup!({ PrintGitHubDiagnostic(diagnostic) }))
            .unwrap();
        markup_to_string(&markup).unwrap()
    }

    #[test]
    fn located_diagnostic_points_at_the_line() {
        let diagnostic = TestDiagnostic {
            path: Some(String::from("migrations/1.sql")),
            span: Some(TextRange::at(TextSize::from(10), TextSize::from(6))),
            source_code: Some(String::from("select 1;\nselect 2;")),
        };
        assert_eq!(
            print(&diagnostic),
            "::error title=internalError/io,file=migrations/1.sql,line=2,endLine=2,col=1,endColumn=7::something failed"
        );
    }

    #[test]
    fn diagnostic_without_source_annotates_the_file() {
        let diagnostic = TestDiagnostic {
            path: Some(String::from("migrations/1.sql")),
            span: None,
            source_code: None,
        };
        assert_eq!(
            print(&diagnostic),
            "::error title=internalError/io,file=migrations/1.sql::something failed"
        );
    }

    #[test]
    fn diagnostic_without_file_annotates_the_run() {
        let diagnostic = TestDiagnostic {
            path: None,
            span: None,
            source_code: None,
        };
        assert_eq!(
            print(&diagnostic),
            "::error title=internalError/io::something failed"
        );
    }
}
