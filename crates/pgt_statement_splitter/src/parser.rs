mod common;
mod data;
mod ddl;
mod dml;

pub use common::source;

use pgt_lexer::{SyntaxKind, Token, WHITESPACE_TOKENS};
use pgt_text_size::{TextRange, TextSize};

use crate::diagnostics::SplitDiagnostic;

/// Main parser that exposes the `cstree` api, and collects errors and statements
/// It is modelled after a Pratt Parser. For a gentle introduction to Pratt Parsing, see https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html
pub struct Parser {
    stmt_ranges: Vec<(usize, usize)>,

    /// The syntax errors accumulated during parsing
    errors: Vec<SplitDiagnostic>,

    /// The start of the current statement, if any
    current_stmt_start: Option<usize>,

    /// The tokens to parse
    pub tokens: Vec<Token>,

    eof_token: Token,

    current_pos: usize,
}

/// Result of Building
#[derive(Debug)]
pub struct Parse {
    /// The ranges of the errors
    pub ranges: Vec<TextRange>,
    /// The syntax errors accumulated during parsing
    pub errors: Vec<SplitDiagnostic>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let eof_token = Token::eof(usize::from(
            tokens
                .last()
                .map(|t| t.span.start())
                .unwrap_or(TextSize::from(0)),
        ));

        // next_pos should be the initialised with the first valid token already
        let mut current_pos = 0;
        while is_irrelevant_token(tokens.get(current_pos).unwrap_or(&eof_token)) {
            current_pos += 1;
        }

        Self {
            stmt_ranges: Vec::new(),
            eof_token,
            errors: Vec::new(),
            current_stmt_start: None,
            tokens,
            current_pos,
        }
    }

    pub fn finish(self) -> Parse {
        Parse {
            ranges: self
                .stmt_ranges
                .iter()
                .map(|(start_token_pos, end_token_pos)| {
                    let from = self.tokens.get(*start_token_pos);
                    let to = self.tokens.get(*end_token_pos).unwrap_or(&self.eof_token);

                    TextRange::new(from.unwrap().span.start(), to.span.end())
                })
                .collect(),
            errors: self.errors,
        }
    }

    /// Start statement
    pub fn start_stmt(&mut self) {
        assert!(
            self.current_stmt_start.is_none(),
            "cannot start statement within statement at {:?}",
            self.tokens.get(self.current_stmt_start.unwrap())
        );
        self.current_stmt_start = Some(self.current_pos);
    }

    /// Close statement
    pub fn close_stmt(&mut self) {
        assert!(
            matches!(self.current_stmt_start, Some(_)),
            "Must start statement before closing it."
        );

        let start_token_pos = self.current_stmt_start.unwrap();

        assert!(
            self.current_pos > start_token_pos,
            "Must close the statement on a token that's later than the start token."
        );

        // find last relevant token before current position
        let (end_token_pos, _) = self
            .tokens
            .iter()
            .enumerate()
            .take(self.current_pos)
            .rfind(|(_, t)| is_relevant(t))
            .unwrap();

        self.stmt_ranges.push((start_token_pos, end_token_pos));

        self.current_stmt_start = None;
    }

    fn advance(&mut self) -> &Token {
        let mut first_relevant_token = None;
        loop {
            let token = self.tokens.get(self.current_pos).unwrap_or(&self.eof_token);

            // we need to continue with next_pos until the next relevant token after we already
            // found the first one
            if is_relevant(token) {
                if let Some(t) = first_relevant_token {
                    return t;
                }
                first_relevant_token = Some(token);
            }

            self.current_pos += 1;
        }
    }

    fn current(&self) -> &Token {
        match self.tokens.get(self.current_pos) {
            Some(token) => token,
            None => &self.eof_token,
        }
    }

    /// Look ahead to the next relevant token
    fn look_ahead(&self) -> Option<&Token> {
        self.tokens
            .iter()
            .skip(self.current_pos + 1)
            .find(|t| is_relevant(t))
    }

    fn look_back(&self) -> Option<&Token> {
        self.tokens
            .iter()
            .take(self.current_pos)
            .rfind(|t| is_relevant(t))
    }

    /// Returns `true` when it advanced, `false` if it didn't
    pub fn advance_if_kind(&mut self, kind: SyntaxKind) -> bool {
        if self.current().kind == kind {
            self.advance();
            true
        } else {
            false
        }
    }

    pub fn expect(&mut self, kind: SyntaxKind) {
        if self.advance_if_kind(kind) {
            return;
        }

        self.errors.push(SplitDiagnostic::new(
            format!("Expected {:#?}", kind),
            self.current().span,
        ));
    }
}

#[cfg(windows)]
/// Returns true if the token is relevant for the paring process
///
/// On windows, a newline is represented by `\r\n` which is two characters.
fn is_irrelevant_token(t: &Token) -> bool {
    WHITESPACE_TOKENS.contains(&t.kind)
        && (t.kind != SyntaxKind::Newline || t.text == "\r\n" || t.text.chars().count() == 1)
}

#[cfg(not(windows))]
/// Returns true if the token is relevant for the paring process
fn is_irrelevant_token(t: &Token) -> bool {
    WHITESPACE_TOKENS.contains(&t.kind)
        && (t.kind != SyntaxKind::Newline || t.text.chars().count() == 1)
}

fn is_relevant(t: &Token) -> bool {
    !is_irrelevant_token(t)
}
