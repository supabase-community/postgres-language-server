use pgt_diagnostics::{Diagnostic, MessageAndDescription};
use pgt_text_size::TextRange;

use crate::SyntaxKind;

/// Internal error type used during lexing
#[derive(Debug, Clone)]
pub struct LexError {
    pub msg: String,
    pub token: u32,
}

/// A specialized diagnostic for lex errors.
#[derive(Clone, Debug, Diagnostic, PartialEq)]
#[diagnostic(category = "syntax", severity = Error)]
pub struct LexDiagnostic {
    /// The location where the error is occurred
    #[location(span)]
    pub span: TextRange,
    #[message]
    #[description]
    pub message: MessageAndDescription,
}

/// Result of lexing a string, providing access to tokens and diagnostics
pub struct Lexed<'a> {
    pub(crate) text: &'a str,
    pub(crate) kind: Vec<SyntaxKind>,
    pub(crate) start: Vec<u32>,
    pub(crate) error: Vec<LexError>,
}

impl<'a> Lexed<'a> {
    /// Returns the number of tokens (excluding EOF)
    pub fn len(&self) -> usize {
        self.kind.len() - 1
    }

    /// Returns true if there are no tokens
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator over token kinds
    pub fn tokens(&self) -> impl Iterator<Item = SyntaxKind> + '_ {
        (0..self.len()).map(move |i| self.kind(i))
    }

    /// Returns the kind of token at the given index
    pub fn kind(&self, idx: usize) -> SyntaxKind {
        assert!(idx < self.len());
        self.kind[idx]
    }

    /// Returns the text range of token at the given index
    pub fn range(&self, idx: usize) -> TextRange {
        let range = self.text_range(idx);
        TextRange::new(
            range.start.try_into().unwrap(),
            range.end.try_into().unwrap(),
        )
    }

    /// Returns the text of token at the given index
    pub fn text(&self, idx: usize) -> &str {
        self.range_text(idx..idx + 1)
    }

    /// Returns all lexing errors with their text ranges
    pub fn errors(&self) -> Vec<LexDiagnostic> {
        self.error
            .iter()
            .map(|it| {
                let range = self.text_range(it.token as usize);
                LexDiagnostic {
                    message: it.msg.as_str().into(),
                    span: TextRange::new(
                        range.start.try_into().unwrap(),
                        range.end.try_into().unwrap(),
                    ),
                }
            })
            .collect()
    }

    pub(crate) fn text_range(&self, i: usize) -> std::ops::Range<usize> {
        assert!(i < self.len());
        let lo = self.start[i] as usize;
        let hi = self.start[i + 1] as usize;
        lo..hi
    }

    fn range_text(&self, r: std::ops::Range<usize>) -> &str {
        assert!(r.start < r.end && r.end <= self.len());
        let lo = self.start[r.start] as usize;
        let hi = self.start[r.end] as usize;
        &self.text[lo..hi]
    }
}
