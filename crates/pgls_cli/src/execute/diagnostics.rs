use pgls_diagnostics::adapters::{IoError, StdError};
use pgls_diagnostics::{Category, Diagnostic, DiagnosticExt, DiagnosticTags, Error, category};
use std::io;

#[derive(Debug, Diagnostic)]
#[diagnostic(category = "internalError/panic", tags(INTERNAL))]
pub(crate) struct PanicDiagnostic {
    #[description]
    #[message]
    pub(crate) message: String,
}

/// A file that `--skip-errors` skipped.
#[derive(Debug, Diagnostic)]
#[diagnostic(
    category = "syntax",
    message = "Skipped file with syntax errors",
    severity = Information
)]
pub(crate) struct SkippedDiagnostic;

/// Whether `diagnostics` contain a syntax error, which makes `--skip-errors` skip the file.
pub(crate) fn has_syntax_errors<'a, D: Diagnostic + 'a>(
    diagnostics: impl IntoIterator<Item = &'a D>,
) -> bool {
    diagnostics
        .into_iter()
        .any(|diagnostic| diagnostic.category() == Some(category!("syntax")))
}

/// Extension trait for turning [Display]-able error types into [TraversalError]
pub(crate) trait ResultExt {
    type Result;
    fn with_file_path_and_code(
        self,
        file_path: String,
        code: &'static Category,
    ) -> Result<Self::Result, Error>;

    #[allow(unused)]
    fn with_file_path_and_code_and_tags(
        self,
        file_path: String,
        code: &'static Category,
        tags: DiagnosticTags,
    ) -> Result<Self::Result, Error>;
}

impl<T, E> ResultExt for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    type Result = T;

    fn with_file_path_and_code_and_tags(
        self,
        file_path: String,
        code: &'static Category,
        diagnostic_tags: DiagnosticTags,
    ) -> Result<Self::Result, Error> {
        self.map_err(move |err| {
            StdError::from(err)
                .with_category(code)
                .with_file_path(file_path)
                .with_tags(diagnostic_tags)
        })
    }

    fn with_file_path_and_code(
        self,
        file_path: String,
        code: &'static Category,
    ) -> Result<Self::Result, Error> {
        self.map_err(move |err| {
            StdError::from(err)
                .with_category(code)
                .with_file_path(file_path)
        })
    }
}

/// Extension trait for turning [io::Error] into [Error]
pub(crate) trait ResultIoExt: ResultExt {
    fn with_file_path(self, file_path: String) -> Result<Self::Result, Error>;
}

impl<T> ResultIoExt for io::Result<T> {
    fn with_file_path(self, file_path: String) -> Result<Self::Result, Error> {
        self.map_err(|error| IoError::from(error).with_file_path(file_path))
    }
}
