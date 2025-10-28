use pgls_diagnostics::{Diagnostic, MessageAndDescription};
use pgls_text_size::TextRange;

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
#[derive(Debug)]
pub struct Lexed<'a> {
    pub(crate) text: &'a str,
    pub(crate) kind: Vec<SyntaxKind>,
    pub(crate) start: Vec<u32>,
    pub(crate) error: Vec<LexError>,
    pub(crate) line_ending_counts: Vec<usize>,
}

impl Lexed<'_> {
    /// Returns the number of tokens
    pub fn len(&self) -> usize {
        self.kind.len()
    }

    /// Returns true if there are no tokens
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator over token kinds
    pub fn tokens(&self) -> impl Iterator<Item = SyntaxKind> + '_ {
        self.kind.iter().copied()
    }

    /// Returns the kind of token at the given index
    pub fn kind(&self, idx: usize) -> SyntaxKind {
        assert!(
            idx < self.len(),
            "expected index < {}, got {}",
            self.len(),
            idx
        );
        self.kind[idx]
    }

    /// Returns the number of line endings in the token at the given index
    pub fn line_ending_count(&self, idx: usize) -> usize {
        assert!(
            idx < self.len(),
            "expected index < {}, got {}",
            self.len(),
            idx
        );
        assert!(self.kind(idx) == SyntaxKind::LINE_ENDING);
        self.line_ending_counts[idx]
    }

    /// Returns the text range of token at the given index
    pub fn range(&self, idx: usize) -> TextRange {
        self.text_range(idx)
    }

    /// Returns the text of token at the given index
    pub fn text(&self, idx: usize) -> &str {
        self.range_text(idx..idx + 1)
    }

    /// Returns all lexing errors with their text ranges
    pub fn errors(&self) -> Vec<LexDiagnostic> {
        self.error
            .iter()
            .map(|it| LexDiagnostic {
                message: it.msg.as_str().into(),
                span: self.text_range(it.token as usize),
            })
            .collect()
    }

    pub(crate) fn text_range(&self, i: usize) -> TextRange {
        assert!(i < self.len() - 1);
        let lo = self.start[i];
        let hi = self.start[i + 1];
        TextRange::new(lo.into(), hi.into())
    }

    fn range_text(&self, r: std::ops::Range<usize>) -> &str {
        assert!(r.start < r.end && r.end <= self.len());
        let lo = self.start[r.start] as usize;
        let hi = self.start[r.end] as usize;
        &self.text[lo..hi]
    }
}
