use pgt_diagnostics::{Diagnostic, MessageAndDescription};
use pgt_text_size::TextRange;

/// A specialized diagnostic for lex errors.
#[derive(Clone, Debug, Diagnostic, PartialEq)]
#[diagnostic(category = "syntax", severity = Error)]
pub struct LexError {
    /// The location where the error is occurred
    #[location(span)]
    pub span: TextRange,
    #[message]
    #[description]
    pub message: MessageAndDescription,
}
