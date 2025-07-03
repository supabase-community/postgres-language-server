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

#[cfg(test)]
mod tests {
    use crate::lex;

    #[test]
    fn finds_lex_errors() {
        // Test with unterminated block comment
        let input = "/* unterminated comment";
        let lexed = lex(input);
        let errors = lexed.errors();

        // Should have error for unterminated block comment
        assert!(!errors.is_empty());
        assert!(errors[0].message.to_string().contains("Missing trailing"));
        assert!(errors[0].span.start() < errors[0].span.end());

        // Test with unterminated string
        let input2 = "SELECT 'unterminated string";
        let lexed2 = lex(input2);
        let errors2 = lexed2.errors();

        // Should have error for unterminated string
        assert!(!errors2.is_empty());
        assert!(errors2[0].message.to_string().contains("Missing trailing"));
    }
}
