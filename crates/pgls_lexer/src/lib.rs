mod codegen;
mod lexed;
mod lexer;

pub use crate::codegen::syntax_kind::SyntaxKind;
pub use crate::lexed::{LexDiagnostic, Lexed};
pub use crate::lexer::Lexer;

/// Lex the input string into tokens and diagnostics
pub fn lex(input: &str) -> Lexed {
    Lexer::new(input).lex()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_lexing() {
        let input = "SELECT * FROM users WHERE id = 1;";
        let lexed = lex(input);

        // Check we have tokens
        assert!(!lexed.is_empty());

        // Iterate over tokens and collect identifiers
        let mut identifiers = Vec::new();
        for (idx, kind) in lexed.tokens().enumerate() {
            if kind == SyntaxKind::IDENT {
                identifiers.push((lexed.text(idx), lexed.range(idx)));
            }
        }

        // Should find at least "users" and "id" as identifiers
        assert!(identifiers.len() >= 2);
    }

    #[test]
    fn test_lexing_with_errors() {
        let input = "SELECT 'unterminated string";
        let lexed = lex(input);

        // Should have tokens
        assert!(!lexed.is_empty());

        // Should have an error for unterminated string
        let errors = lexed.errors();
        assert!(!errors.is_empty());
        // Check the error message exists
        assert!(!errors[0].message.to_string().is_empty());
    }

    #[test]
    fn test_lexing_string_params_with_errors() {
        let input = "SELECT :'unterminated string";
        let lexed = lex(input);

        // Should have tokens
        assert!(!lexed.is_empty());

        // Should have an error for unterminated string
        let errors = lexed.errors();
        assert!(!errors.is_empty());
        // Check the error message exists
        assert!(!errors[0].message.to_string().is_empty());
    }

    #[test]
    fn test_lexing_identifier_params_with_errors() {
        let input = "SELECT :\"unterminated string";
        let lexed = lex(input);

        // Should have tokens
        assert!(!lexed.is_empty());

        // Should have an error for unterminated string
        let errors = lexed.errors();
        assert!(!errors.is_empty());
        // Check the error message exists
        assert!(!errors[0].message.to_string().is_empty());
    }

    #[test]
    fn test_token_ranges() {
        let input = "SELECT id";
        let lexed = lex(input);

        // First token should be a keyword (SELECT gets parsed as a keyword)
        let _first_kind = lexed.kind(0);
        assert_eq!(u32::from(lexed.range(0).start()), 0);
        assert_eq!(u32::from(lexed.range(0).end()), 6);
        assert_eq!(lexed.text(0), "SELECT");

        // Find the id token
        for (idx, kind) in lexed.tokens().enumerate() {
            if kind == SyntaxKind::IDENT && lexed.text(idx) == "id" {
                assert_eq!(u32::from(lexed.range(idx).start()), 7);
                assert_eq!(u32::from(lexed.range(idx).end()), 9);
            }
        }
    }

    #[test]
    fn test_empty_input() {
        let input = "";
        let lexed = lex(input);
        assert_eq!(lexed.len(), 1);
        assert_eq!(lexed.kind(0), SyntaxKind::EOF);
    }

    #[test]
    fn test_whitespace_handling() {
        let input = "  SELECT  \n  id  ";
        let lexed = lex(input);

        // Collect non-whitespace tokens
        let mut non_whitespace = Vec::new();
        for (idx, kind) in lexed.tokens().enumerate() {
            if !matches!(
                kind,
                SyntaxKind::SPACE | SyntaxKind::TAB | SyntaxKind::LINE_ENDING | SyntaxKind::EOF
            ) {
                non_whitespace.push(lexed.text(idx));
            }
        }

        assert_eq!(non_whitespace.len(), 2); // SELECT and id
    }

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
