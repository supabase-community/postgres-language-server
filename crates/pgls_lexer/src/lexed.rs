use pgls_diagnostics::{Diagnostic, MessageAndDescription};
use pgls_text_size::{TextRange, TextSize};

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
    pub(crate) has_blank_line: bool,
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

    /// Returns the kind of token at the given index, `EOF` past the last token
    pub fn kind(&self, idx: usize) -> SyntaxKind {
        self.kind.get(idx).copied().unwrap_or(SyntaxKind::EOF)
    }

    /// Returns the number of line endings in the token at the given index, 0 if it isn't a line
    /// ending
    pub fn line_ending_count(&self, idx: usize) -> usize {
        debug_assert!(self.kind(idx) == SyntaxKind::LINE_ENDING);
        self.line_ending_counts.get(idx).copied().unwrap_or(0)
    }

    /// Returns whether the source contains at least one blank line.
    pub fn has_blank_line(&self) -> bool {
        self.has_blank_line
    }

    /// Returns the text range of token at the given index, an empty range at the end of the text
    /// for `EOF` and past the last token
    pub fn range(&self, idx: usize) -> TextRange {
        self.text_range(idx)
    }

    /// Returns the text of token at the given index, empty for `EOF` and past the last token
    pub fn text(&self, idx: usize) -> &str {
        let range = self.text_range(idx);
        self.text
            .get(usize::from(range.start())..usize::from(range.end()))
            .unwrap_or_default()
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

    /// A token ends where the next one starts, and `EOF` at the end of the text.
    pub(crate) fn text_range(&self, i: usize) -> TextRange {
        let end = TextSize::of(self.text);
        let start = |i: usize| self.start.get(i).map_or(end, |&start| start.into());
        TextRange::new(start(i), start(i + 1))
    }
}
