use pgls_diagnostics::{Diagnostic, MessageAndDescription};
use pgls_lexer::{LexDiagnostic, Lexed};
use pgls_text_size::TextRange;

use crate::splitter::SplitError;

/// A specialized diagnostic for the statement splitter parser.
///
/// Parser diagnostics are always **errors**.
#[derive(Clone, Debug, Diagnostic, PartialEq)]
#[diagnostic(category = "syntax", severity = Error)]
pub struct SplitDiagnostic {
    /// The location where the error is occurred
    #[location(span)]
    span: Option<TextRange>,
    #[message]
    #[description]
    pub message: MessageAndDescription,
}

impl SplitDiagnostic {
    pub fn new(message: impl Into<String>, range: TextRange) -> Self {
        Self {
            span: Some(range),
            message: MessageAndDescription::from(message.into()),
        }
    }
}

impl From<LexDiagnostic> for SplitDiagnostic {
    fn from(lex_diagnostic: LexDiagnostic) -> Self {
        Self {
            span: Some(lex_diagnostic.span),
            message: lex_diagnostic.message,
        }
    }
}

impl SplitDiagnostic {
    pub fn from_split_error(split_error: SplitError, lexed: &Lexed) -> Self {
        let range = lexed.range(split_error.token);
        Self {
            span: Some(range),
            message: MessageAndDescription::from(split_error.msg),
        }
    }
}
