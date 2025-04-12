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
    /// The ranges of the statements
    ranges: Vec<(usize, usize)>,
    /// The syntax errors accumulated during parsing
    errors: Vec<SplitDiagnostic>,
    /// The start of the current statement, if any
    current_stmt_start: Option<usize>,
    /// The tokens to parse
    pub tokens: Vec<Token>,

    eof_token: Token,

    next_pos: usize,
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
        let mut next_pos = 0;
        loop {
            let token = tokens.get(next_pos).unwrap_or(&eof_token);

            if is_irrelevant_token(token) {
                next_pos += 1;
            } else {
                break;
            }
        }

        Self {
            ranges: Vec::new(),
            eof_token,
            errors: Vec::new(),
            current_stmt_start: None,
            tokens,
            next_pos,
        }
    }

    pub fn finish(self) -> Parse {
        Parse {
            ranges: self
                .ranges
                .iter()
                .map(|(start, end)| {
                    let from = self.tokens.get(*start);
                    let to = self.tokens.get(*end).unwrap_or(&self.eof_token);

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
        self.current_stmt_start = Some(self.next_pos);
    }

    /// Close statement
    pub fn close_stmt(&mut self) {
        assert!(self.next_pos > 0);

        // go back the positions until we find the first relevant token
        let mut end_token_pos = self.next_pos - 1;
        loop {
            let token = self.tokens.get(end_token_pos);

            if end_token_pos == 0 || token.is_none() {
                break;
            }

            if !is_irrelevant_token(token.unwrap()) {
                break;
            }

            end_token_pos -= 1;
        }

        self.ranges.push((
            self.current_stmt_start.expect("Expected active statement"),
            end_token_pos,
        ));

        self.current_stmt_start = None;
    }

    fn advance(&mut self) -> &Token {
        let mut first_relevant_token = None;
        loop {
            let token = self.tokens.get(self.next_pos).unwrap_or(&self.eof_token);

            // we need to continue with next_pos until the next relevant token after we already
            // found the first one
            if !is_irrelevant_token(token) {
                if let Some(t) = first_relevant_token {
                    return t;
                }
                first_relevant_token = Some(token);
            }

            self.next_pos += 1;
        }
    }

    fn peek(&self) -> &Token {
        match self.tokens.get(self.next_pos) {
            Some(token) => token,
            None => &self.eof_token,
        }
    }

    /// Look ahead to the next relevant token
    /// Returns `None` if we are already at the last relevant token
    fn look_ahead(&self) -> Option<&Token> {
        // we need to look ahead to the next relevant token
        let mut look_ahead_pos = self.next_pos + 1;
        loop {
            let token = self.tokens.get(look_ahead_pos)?;

            if !is_irrelevant_token(token) {
                return Some(token);
            }

            look_ahead_pos += 1;
        }
    }

    /// Returns `None` if there are no previous relevant tokens
    fn look_back(&self) -> Option<&Token> {
        // we need to look back to the last relevant token
        let mut look_back_pos = self.next_pos - 1;
        loop {
            let token = self.tokens.get(look_back_pos);

            if look_back_pos == 0 || token.is_none() {
                return None;
            }

            if !is_irrelevant_token(token.unwrap()) {
                return token;
            }

            look_back_pos -= 1;
        }
    }

    /// checks if the current token is of `kind` and advances if true
    /// returns true if the current token is of `kind`
    pub fn eat(&mut self, kind: SyntaxKind) -> bool {
        if self.peek().kind == kind {
            self.advance();
            true
        } else {
            false
        }
    }

    pub fn expect(&mut self, kind: SyntaxKind) {
        if self.eat(kind) {
            return;
        }

        self.errors.push(SplitDiagnostic::new(
            format!("Expected {:#?}", kind),
            self.peek().span,
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
