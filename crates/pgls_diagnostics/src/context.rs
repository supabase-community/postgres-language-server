use pgls_console::fmt;

use crate::{
    Category, DiagnosticTags, Error, Resource, Severity, SourceCode,
    diagnostic::internal::AsDiagnostic,
    location::{AsResource, AsSourceCode, AsSpan},
};

/// This trait is implemented for all types implementing [Diagnostic](super::Diagnostic)
/// and the [Error] struct, and exposes various combinator methods to enrich
/// existing diagnostics with additional information.
pub trait DiagnosticExt: internal::Sealed + Sized {
    /// Returns a new diagnostic with the provided `message` as a message and
    /// description, and `self` as a source diagnostic. This is useful to
    /// create chains of diagnostics, where high level errors wrap lower level
    /// causes.
    fn context<M>(self, message: M) -> Error
    where
        Self: 'static,
        M: fmt::Display + 'static,
        Error: From<internal::ContextDiagnostic<M, Self>>;

    /// Returns a new diagnostic using the provided `category` if `self`
    /// doesn't already have one.
    fn with_category(self, category: &'static Category) -> Error
    where
        Error: From<internal::Decorated<Self>>;

    /// Returns a new diagnostic using the provided `path` if `self`
    /// doesn't already have one.
    fn with_file_path(self, path: impl AsResource) -> Error
    where
        Error: From<internal::Decorated<Self>>;

    /// Returns a new diagnostic using the provided `span` instead of the one in `self`.
    fn with_file_span(self, span: impl AsSpan) -> Error
    where
        Error: From<internal::Decorated<Self>>;

    /// Returns a new diagnostic using the provided `source_code` if `self`
    /// doesn't already have one.
    fn with_file_source_code(self, source_code: impl AsSourceCode) -> Error
    where
        Error: From<internal::Decorated<Self>>;

    /// Returns a new diagnostic with additional `tags`
    fn with_tags(self, tags: DiagnosticTags) -> Error
    where
        Error: From<internal::Decorated<Self>>;

    /// Returns a new diagnostic with additional `severity`
    fn with_severity(self, severity: Severity) -> Error
    where
        Error: From<internal::Decorated<Self>>;
}

impl<E: AsDiagnostic> internal::Sealed for E {}

impl<E: AsDiagnostic> DiagnosticExt for E {
    fn context<M>(self, message: M) -> Error
    where
        E: 'static,
        M: fmt::Display + 'static,
        Error: From<internal::ContextDiagnostic<M, E>>,
    {
        Error::from(internal::ContextDiagnostic {
            message,
            source: self,
        })
    }

    fn with_category(self, category: &'static Category) -> Error
    where
        Error: From<internal::Decorated<Self>>,
    {
        Error::from(internal::Decorated {
            category: Some(category),
            ..internal::Decorated::new(self)
        })
    }

    fn with_file_path(self, path: impl AsResource) -> Error
    where
        Error: From<internal::Decorated<E>>,
    {
        Error::from(internal::Decorated {
            path: path.as_resource().map(Resource::to_owned),
            ..internal::Decorated::new(self)
        })
    }

    fn with_file_span(self, span: impl AsSpan) -> Error
    where
        Error: From<internal::Decorated<E>>,
    {
        Error::from(internal::Decorated {
            span: span.as_span(),
            ..internal::Decorated::new(self)
        })
    }

    fn with_file_source_code(self, source_code: impl AsSourceCode) -> Error
    where
        Error: From<internal::Decorated<Self>>,
    {
        Error::from(internal::Decorated {
            source_code: source_code.as_source_code().map(SourceCode::to_owned),
            ..internal::Decorated::new(self)
        })
    }

    fn with_tags(self, tags: DiagnosticTags) -> Error
    where
        Error: From<internal::Decorated<Self>>,
    {
        Error::from(internal::Decorated {
            tags,
            ..internal::Decorated::new(self)
        })
    }

    fn with_severity(self, severity: Severity) -> Error
    where
        Error: From<internal::Decorated<Self>>,
    {
        Error::from(internal::Decorated {
            severity: Some(severity),
            ..internal::Decorated::new(self)
        })
    }
}

pub trait Context<T, E>: internal::Sealed {
    /// If `self` is an error, returns a new diagnostic with the provided
    /// `message` as a message and description, and `self` as a source
    /// diagnostic. This is useful to create chains of diagnostics, where high
    /// level errors wrap lower level causes.
    fn context<M>(self, message: M) -> Result<T, Error>
    where
        E: 'static,
        M: fmt::Display + 'static,
        Error: From<internal::ContextDiagnostic<M, E>>;

    /// If `self` is an error, returns a new diagnostic using the provided
    /// `category` if `self` doesn't already have one.
    fn with_category(self, category: &'static Category) -> Result<T, Error>
    where
        Error: From<internal::Decorated<E>>;

    /// If `self` is an error, returns a new diagnostic using the provided
    /// `path` if `self` doesn't already have one.
    fn with_file_path(self, path: impl AsResource) -> Result<T, Error>
    where
        Error: From<internal::Decorated<E>>;

    /// If `self` is an error, returns a new diagnostic using the provided
    /// `severity` if `self` doesn't already have one.
    fn with_severity(self, severity: Severity) -> Result<T, Error>
    where
        Error: From<internal::Decorated<E>>;

    /// If `self` is an error, returns a new diagnostic using the provided
    /// `tags` if `self` doesn't already have one.
    fn with_tags(self, tags: DiagnosticTags) -> Result<T, Error>
    where
        Error: From<internal::Decorated<E>>;

    /// If `self` is an error, returns a new diagnostic using the provided
    /// `span` instead of the one returned by `self`.
    ///
    /// This is useful in multi-language documents, where a given diagnostic
    /// may be originally emitted with a span relative to a specific substring
    /// of a larger document, and later needs to have its position remapped to
    /// be relative to the entire file instead.
    fn with_file_span(self, span: impl AsSpan) -> Result<T, Error>
    where
        Error: From<internal::Decorated<E>>;
}

impl<T, E: AsDiagnostic> internal::Sealed for Result<T, E> {}

impl<T, E: AsDiagnostic> Context<T, E> for Result<T, E> {
    fn context<M>(self, message: M) -> Result<T, Error>
    where
        E: 'static,
        M: fmt::Display + 'static,
        Error: From<internal::ContextDiagnostic<M, E>>,
    {
        match self {
            Ok(value) => Ok(value),
            Err(source) => Err(source.context(message)),
        }
    }

    fn with_category(self, category: &'static Category) -> Result<T, Error>
    where
        Error: From<internal::Decorated<E>>,
    {
        match self {
            Ok(value) => Ok(value),
            Err(source) => Err(source.with_category(category)),
        }
    }

    fn with_file_path(self, path: impl AsResource) -> Result<T, Error>
    where
        Error: From<internal::Decorated<E>>,
    {
        match self {
            Ok(value) => Ok(value),
            Err(source) => Err(source.with_file_path(path)),
        }
    }

    fn with_severity(self, severity: Severity) -> Result<T, Error>
    where
        Error: From<internal::Decorated<E>>,
    {
        match self {
            Ok(value) => Ok(value),
            Err(source) => Err(source.with_severity(severity)),
        }
    }

    fn with_tags(self, tags: DiagnosticTags) -> Result<T, Error>
    where
        Error: From<internal::Decorated<E>>,
    {
        match self {
            Ok(value) => Ok(value),
            Err(source) => Err(source.with_tags(tags)),
        }
    }

    fn with_file_span(self, span: impl AsSpan) -> Result<T, Error>
    where
        Error: From<internal::Decorated<E>>,
    {
        match self {
            Ok(value) => Ok(value),
            Err(source) => Err(source.with_file_span(span)),
        }
    }
}

mod internal {
    //! These types need to be declared as public as they're referred to in the
    //! `where` clause of other public items, but as they're not part of the
    //! public API they are declared in a private module so they're not
    //! accessible outside of the crate

    use std::{fmt::Debug, io};

    use pgls_console::{fmt, markup};
    use pgls_text_edit::TextEdit;
    use pgls_text_size::TextRange;

    use crate::{
        Advices, Backtrace, Category, Diagnostic, DiagnosticTags, LineIndex, LineIndexBuf,
        Location, LogCategory, Resource, Severity, SourceCode, Visit,
        diagnostic::internal::AsDiagnostic,
    };

    /// This trait is inherited by `DiagnosticExt` and `Context`, since it's
    /// not visible outside of `pgls_diagnostics` this prevents these extension
    /// traits from being implemented on other types outside of this module
    ///
    /// Making these traits "sealed" is mainly intended as a stability
    /// guarantee, if these traits were simply public any change to their
    /// signature or generic implementations would be a breaking change for
    /// downstream implementations, so preventing these traits from ever being
    /// implemented in downstream crates ensures this doesn't happen.
    pub trait Sealed {}

    /// Diagnostic type returned by [super::DiagnosticExt::context], uses
    /// `message` as its message and description, and `source` as its source
    /// diagnostic.
    pub struct ContextDiagnostic<M, E> {
        pub(super) message: M,
        pub(super) source: E,
    }

    impl<M: fmt::Display + 'static, E: AsDiagnostic> Diagnostic for ContextDiagnostic<M, E> {
        fn category(&self) -> Option<&'static Category> {
            self.source.as_diagnostic().category()
        }

        fn severity(&self) -> Severity {
            self.source.as_diagnostic().severity()
        }

        fn description(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut writer = DisplayMarkup(fmt);
            let mut fmt = fmt::Formatter::new(&mut writer);
            fmt.write_markup(markup!({ self.message }))
                .map_err(|_| std::fmt::Error)
        }

        fn message(&self, fmt: &mut fmt::Formatter<'_>) -> io::Result<()> {
            fmt::Display::fmt(&self.message, fmt)
        }

        fn advices(&self, visitor: &mut dyn Visit) -> io::Result<()> {
            self.source.as_diagnostic().advices(visitor)
        }

        fn verbose_advices(&self, visitor: &mut dyn Visit) -> io::Result<()> {
            self.source.as_diagnostic().verbose_advices(visitor)
        }

        fn location(&self) -> Location<'_> {
            self.source.as_diagnostic().location()
        }

        fn tags(&self) -> DiagnosticTags {
            self.source.as_diagnostic().tags()
        }

        fn source(&self) -> Option<&dyn Diagnostic> {
            Some(self.source.as_dyn())
        }
    }

    impl<M: fmt::Display, E: Debug> Debug for ContextDiagnostic<M, E> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Diagnostic")
                .field("message", &DebugMarkup(&self.message))
                .field("source", &self.source)
                .finish()
        }
    }

    /// Helper wrapper implementing [Debug] for types implementing [fmt::Display],
    /// prints a debug representation of the markup generated by printing `T`.
    struct DebugMarkup<T>(T);

    impl<T: fmt::Display> Debug for DebugMarkup<T> {
        fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let buffer = markup!({ self.0 }).to_owned();
            Debug::fmt(&buffer, fmt)
        }
    }

    /// Helper wrapper implementing [fmt::Write] for [std::fmt::Formatter].
    struct DisplayMarkup<'a, 'b>(&'a mut std::fmt::Formatter<'b>);

    impl fmt::Write for DisplayMarkup<'_, '_> {
        fn write_str(&mut self, _: &fmt::MarkupElements<'_>, content: &str) -> io::Result<()> {
            self.0.write_str(content).map_err(io::Error::other)
        }

        fn write_fmt(
            &mut self,
            _: &fmt::MarkupElements<'_>,
            content: std::fmt::Arguments<'_>,
        ) -> io::Result<()> {
            self.0.write_fmt(content).map_err(io::Error::other)
        }
    }

    /// Diagnostic type returned by the `with_*` methods of [super::DiagnosticExt]: `source`
    /// with the given overrides. Everything that isn't overridden forwards to `source`,
    /// [Diagnostic::source] included, so a chain of causes survives a category, path, span,
    /// source code, tags or severity added on top of it.
    pub struct Decorated<E> {
        /// Used if `source` has no category.
        pub(super) category: Option<&'static Category>,
        /// Used if `source` has no resource, and replaces the path of a file.
        pub(super) path: Option<Resource<String>>,
        /// Replaces the span of `source`.
        pub(super) span: Option<TextRange>,
        /// Used if `source` has no source code, also for the code frames of its advices.
        pub(super) source_code: Option<SourceCode<String, LineIndexBuf>>,
        /// Added to the tags of `source`.
        pub(super) tags: DiagnosticTags,
        /// Replaces the severity of `source`.
        pub(super) severity: Option<Severity>,
        pub(super) source: E,
    }

    impl<E> Decorated<E> {
        /// `source` without overrides.
        pub(super) fn new(source: E) -> Self {
            Self {
                category: None,
                path: None,
                span: None,
                source_code: None,
                tags: DiagnosticTags::empty(),
                severity: None,
                source,
            }
        }
    }

    impl<E: Debug> Debug for Decorated<E> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut debug = f.debug_struct("Diagnostic");
            if let Some(category) = &self.category {
                debug.field("category", category);
            }
            if let Some(path) = &self.path {
                debug.field("path", path);
            }
            if let Some(span) = &self.span {
                debug.field("span", span);
            }
            if let Some(source_code) = &self.source_code {
                debug.field("source_code", source_code);
            }
            if !self.tags.is_empty() {
                debug.field("tags", &self.tags);
            }
            if let Some(severity) = &self.severity {
                debug.field("severity", severity);
            }
            debug.field("source", &self.source).finish()
        }
    }

    impl<E: AsDiagnostic> Diagnostic for Decorated<E> {
        fn category(&self) -> Option<&'static Category> {
            self.source.as_diagnostic().category().or(self.category)
        }

        fn severity(&self) -> Severity {
            self.severity
                .unwrap_or_else(|| self.source.as_diagnostic().severity())
        }

        fn description(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.source.as_diagnostic().description(fmt)
        }

        fn message(&self, fmt: &mut fmt::Formatter<'_>) -> io::Result<()> {
            self.source.as_diagnostic().message(fmt)
        }

        fn advices(&self, visitor: &mut dyn Visit) -> io::Result<()> {
            if let Some(source_code) = &self.source_code {
                let mut visitor = FileSourceCodeVisitor {
                    visitor,
                    source_code: source_code.as_deref(),
                };

                self.source.as_diagnostic().advices(&mut visitor)
            } else {
                self.source.as_diagnostic().advices(visitor)
            }
        }

        fn verbose_advices(&self, visitor: &mut dyn Visit) -> io::Result<()> {
            if let Some(source_code) = &self.source_code {
                let mut visitor = FileSourceCodeVisitor {
                    visitor,
                    source_code: source_code.as_deref(),
                };

                self.source.as_diagnostic().verbose_advices(&mut visitor)
            } else {
                self.source.as_diagnostic().verbose_advices(visitor)
            }
        }

        fn location(&self) -> Location<'_> {
            let location = self.source.as_diagnostic().location();
            Location {
                resource: match (location.resource, &self.path) {
                    (Some(Resource::File(_)), Some(Resource::File(path))) => {
                        Some(Resource::File(path.as_ref()))
                    }
                    (None, path) => path.as_ref().map(Resource::as_deref),
                    (resource, _) => resource,
                },
                span: self.span.or(location.span),
                source_code: location
                    .source_code
                    .or_else(|| Some(self.source_code.as_ref()?.as_deref())),
            }
        }

        fn tags(&self) -> DiagnosticTags {
            self.source.as_diagnostic().tags() | self.tags
        }

        fn source(&self) -> Option<&dyn Diagnostic> {
            self.source.as_diagnostic().source()
        }
    }

    /// Helper wrapper for a [Visitor], automatically inject `source_code` into
    /// the location of code frame advices if they don't have one already.
    struct FileSourceCodeVisitor<'a> {
        visitor: &'a mut dyn Visit,
        source_code: SourceCode<&'a str, &'a LineIndex>,
    }

    impl Visit for FileSourceCodeVisitor<'_> {
        fn record_log(&mut self, category: LogCategory, text: &dyn fmt::Display) -> io::Result<()> {
            self.visitor.record_log(category, text)
        }

        fn record_list(&mut self, list: &[&dyn fmt::Display]) -> io::Result<()> {
            self.visitor.record_list(list)
        }

        fn record_frame(&mut self, location: Location<'_>) -> io::Result<()> {
            self.visitor.record_frame(Location {
                source_code: Some(location.source_code.unwrap_or(self.source_code)),
                ..location
            })
        }

        fn record_diff(&mut self, diff: &TextEdit) -> io::Result<()> {
            self.visitor.record_diff(diff)
        }

        fn record_diff_with_offset(&mut self, diff: &TextEdit, line_offset: u32) -> io::Result<()> {
            self.visitor.record_diff_with_offset(diff, line_offset)
        }

        fn record_backtrace(
            &mut self,
            title: &dyn fmt::Display,
            backtrace: &Backtrace,
        ) -> io::Result<()> {
            self.visitor.record_backtrace(title, backtrace)
        }

        fn record_command(&mut self, command: &str) -> io::Result<()> {
            self.visitor.record_command(command)
        }

        fn record_group(
            &mut self,
            title: &dyn fmt::Display,
            advice: &dyn Advices,
        ) -> io::Result<()> {
            self.visitor.record_group(title, advice)
        }
    }
}

#[cfg(test)]
mod tests {
    use pgls_text_size::{TextRange, TextSize};

    use crate::{self as pgls_diagnostics, DiagnosticExt, DiagnosticTags, Resource, Severity};
    use pgls_diagnostics_macros::Diagnostic;

    #[derive(Debug, Diagnostic)]
    #[diagnostic(category = "internalError/io", severity = Error, message = "the cause")]
    struct Cause;

    #[test]
    fn decorations_keep_the_cause() {
        let span = TextRange::new(TextSize::from(1), TextSize::from(4));
        let error = Cause
            .context("the error")
            .with_category(pgls_diagnostics::category!("internalError/fs"))
            .with_file_path("file.sql")
            .with_file_span(span)
            .with_file_source_code("select 1;")
            .with_tags(DiagnosticTags::VERBOSE)
            .with_severity(Severity::Warning);

        let cause = error.source().expect("the cause survives the decorations");
        assert_eq!(cause.category().map(|c| c.name()), Some("internalError/io"));

        // The context takes its category from the cause, so the decoration's is unused.
        assert_eq!(error.category().map(|c| c.name()), Some("internalError/io"));
        assert_eq!(error.severity(), Severity::Warning);
        assert!(error.tags().is_verbose());
        let location = error.location();
        assert_eq!(location.resource, Some(Resource::File("file.sql")));
        assert_eq!(location.span, Some(span));
        assert_eq!(location.source_code.map(|code| code.text), Some("select 1;"));
    }
}
