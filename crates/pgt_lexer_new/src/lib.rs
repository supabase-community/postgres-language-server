mod codegen;
mod diagnostics;
mod lexed_str;

use diagnostics::LexError;
use lexed_str::LexedStr;
use pgt_text_size::TextRange;

pub use crate::codegen::syntax_kind::SyntaxKind;

/// Result of lexing a string, providing access to tokens and diagnostics
pub struct Lexed<'a> {
    inner: LexedStr<'a>,
}

impl Lexed<'_> {
    /// Returns the number of tokens (excluding EOF)
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns true if there are no tokens
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator over token kinds
    pub fn tokens(&self) -> impl Iterator<Item = SyntaxKind> + '_ {
        (0..self.len()).map(move |i| self.inner.kind(i))
    }

    /// Returns the kind of token at the given index
    pub fn kind(&self, idx: usize) -> SyntaxKind {
        self.inner.kind(idx)
    }

    /// Returns the text range of token at the given index
    pub fn range(&self, idx: usize) -> TextRange {
        let range = self.inner.text_range(idx);
        TextRange::new(
            range.start.try_into().unwrap(),
            range.end.try_into().unwrap(),
        )
    }

    /// Returns the text of token at the given index
    pub fn text(&self, idx: usize) -> &str {
        self.inner.text(idx)
    }

    /// Returns all lexing errors with their text ranges
    pub fn errors(&self) -> Vec<LexError> {
        self.inner
            .errors()
            .map(|(i, msg)| {
                let range = self.inner.text_range(i);
                LexError {
                    message: msg.into(),
                    span: TextRange::new(
                        range.start.try_into().unwrap(),
                        range.end.try_into().unwrap(),
                    ),
                }
            })
            .collect()
    }
}

/// Lex the input string into tokens and diagnostics
pub fn lex(input: &str) -> Lexed {
    Lexed {
        inner: LexedStr::new(input),
    }
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
        assert!(lexed.is_empty());
        assert_eq!(lexed.len(), 0);
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
                SyntaxKind::SPACE | SyntaxKind::TAB | SyntaxKind::NEWLINE
            ) {
                non_whitespace.push(lexed.text(idx));
            }
        }

        assert_eq!(non_whitespace.len(), 2); // SELECT and id
    }
}
