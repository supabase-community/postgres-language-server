use pgls_diagnostics::{Diagnostic, MessageAndDescription};
use pgls_text_size::TextRange;

/// A specialized diagnostic for the libpg_query parser.
///
/// Parser diagnostics are always **errors**.
#[derive(Clone, Debug, Diagnostic)]
#[diagnostic(category = "syntax", severity = Error)]
pub struct SyntaxDiagnostic {
    /// The location where the error is occurred
    #[location(span)]
    pub span: Option<TextRange>,
    #[message]
    #[description]
    pub message: MessageAndDescription,
}

impl SyntaxDiagnostic {
    /// Create a new syntax diagnostic with the given message and optional span.
    pub fn new(message: impl Into<String>, span: Option<TextRange>) -> Self {
        SyntaxDiagnostic {
            span,
            message: MessageAndDescription::from(message.into()),
        }
    }

    pub fn span(mut self, span: TextRange) -> Self {
        self.span = Some(span);
        self
    }
}

impl From<pgls_query::Error> for SyntaxDiagnostic {
    fn from(err: pgls_query::Error) -> Self {
        SyntaxDiagnostic {
            span: None,
            message: MessageAndDescription::from(err.to_string()),
        }
    }
}
