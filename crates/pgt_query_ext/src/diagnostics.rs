use pgt_diagnostics::{Diagnostic, MessageAndDescription};
use pgt_text_size::TextRange;

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

impl From<pg_query::Error> for SyntaxDiagnostic {
    fn from(err: pg_query::Error) -> Self {
        SyntaxDiagnostic {
            span: None,
            message: MessageAndDescription::from(err.to_string()),
        }
    }
}
