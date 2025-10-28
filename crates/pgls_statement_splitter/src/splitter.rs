mod common;
mod data;
mod ddl;
mod dml;

pub use common::source;

use pgls_lexer::{Lexed, SyntaxKind};
use pgls_text_size::TextRange;

use crate::splitter::common::{ReachedEOFException, SplitterResult};

pub struct SplitResult {
    pub ranges: Vec<TextRange>,
    pub errors: Vec<SplitError>,
}

pub static TRIVIA_TOKENS: &[SyntaxKind] = &[
    SyntaxKind::SPACE,
    SyntaxKind::TAB,
    SyntaxKind::VERTICAL_TAB,
    SyntaxKind::FORM_FEED,
    SyntaxKind::COMMENT,
    // LINE_ENDING is relevant
];

/// Internal error type used during splitting
#[derive(Debug, Clone)]
pub struct SplitError {
    pub msg: String,
    pub token: usize,
}

#[derive(Debug)]
pub struct Splitter<'a> {
    lexed: &'a Lexed<'a>,
    current_pos: usize,
    stmt_ranges: Vec<(usize, usize)>,
    errors: Vec<SplitError>,
    current_stmt_start: Option<usize>,
}

impl<'a> Splitter<'a> {
    pub fn new(lexed: &'a Lexed<'a>) -> Self {
        Self {
            lexed,
            current_pos: 0,
            stmt_ranges: Vec::new(),
            errors: Vec::new(),
            current_stmt_start: None,
        }
    }

    pub fn finish(self) -> SplitResult {
        let ranges = self
            .stmt_ranges
            .iter()
            .map(|(start_token_pos, end_token_pos)| {
                let from = self.lexed.range(*start_token_pos).start();
                let to = self.lexed.range(*end_token_pos).end();
                TextRange::new(from, to)
            })
            .collect();

        SplitResult {
            ranges,
            errors: self.errors,
        }
    }

    pub fn start_stmt(&mut self) {
        assert!(
            self.current_stmt_start.is_none(),
            "cannot start statement within statement",
        );
        self.current_stmt_start = Some(self.current_pos);
    }

    pub fn close_stmt(&mut self) {
        assert!(
            self.current_stmt_start.is_some(),
            "Must start statement before closing it."
        );

        let start_token_pos = self.current_stmt_start.unwrap();

        assert!(
            self.current_pos > start_token_pos,
            "Must close the statement on a token that's later than the start token: {} > {}",
            self.current_pos,
            start_token_pos
        );

        let end_token_pos = (0..self.current_pos)
            .rev()
            .find(|&idx| !self.is_trivia(idx))
            .unwrap();

        self.stmt_ranges.push((start_token_pos, end_token_pos));

        self.current_stmt_start = None;
    }

    fn current(&self) -> SyntaxKind {
        self.lexed.kind(self.current_pos)
    }

    fn eat(&mut self, kind: SyntaxKind) -> Result<bool, ReachedEOFException> {
        if self.current() == kind {
            self.advance()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn kind(&self, idx: usize) -> SyntaxKind {
        self.lexed.kind(idx)
    }

    /// Advances the parser to the next relevant token and returns it.
    ///
    /// NOTE: This will skip trivia tokens.
    fn advance(&mut self) -> Result<SyntaxKind, ReachedEOFException> {
        if self.current() == SyntaxKind::EOF {
            return Err(ReachedEOFException);
        }

        let pos = (self.current_pos + 1..self.lexed.len())
            .find(|&idx| !self.is_trivia(idx))
            .unwrap();

        self.current_pos = pos;
        Ok(self.lexed.kind(pos))
    }

    fn look_ahead(&self, ignore_trivia: bool) -> SyntaxKind {
        let pos = if ignore_trivia {
            (self.current_pos + 1..self.lexed.len())
                .find(|&idx| !self.is_trivia(idx))
                .expect("lexed should have non-trivia eof token")
        } else {
            (self.current_pos + 1..self.lexed.len())
                .next()
                .expect("lexed should have a eof token")
        };
        self.lexed.kind(pos)
    }

    /// Returns `None` if there are no previous relevant tokens
    fn look_back(&self, ignore_trivia: bool) -> Option<SyntaxKind> {
        if ignore_trivia {
            (0..self.current_pos)
                .rev()
                .find(|&idx| !self.is_trivia(idx))
                .map(|idx| self.lexed.kind(idx))
        } else {
            (0..self.current_pos)
                .next_back()
                .map(|idx| self.lexed.kind(idx))
        }
    }

    fn is_trivia(&self, idx: usize) -> bool {
        match self.lexed.kind(idx) {
            k if TRIVIA_TOKENS.contains(&k) => true,
            SyntaxKind::LINE_ENDING => self.lexed.line_ending_count(idx) < 2,
            _ => false,
        }
    }

    /// Will advance if the `kind` matches the current token.
    /// Otherwise, will add a diagnostic to the internal `errors`.
    fn expect(&mut self, kind: SyntaxKind) -> SplitterResult {
        if self.current() == kind {
            self.advance()?;
        } else {
            let token = if self.current() == SyntaxKind::EOF {
                self.current_pos - 1
            } else {
                self.current_pos
            };

            self.errors.push(SplitError {
                msg: format!("Expected {kind:#?}"),
                token,
            });
        };

        Ok(())
    }
}
