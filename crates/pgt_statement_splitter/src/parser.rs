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
    /// The statement ranges are defined by the indices of the start/end tokens
    stmt_ranges: Vec<(usize, usize)>,

    /// The syntax errors accumulated during parsing
    errors: Vec<SplitDiagnostic>,

    current_stmt_start: Option<usize>,

    tokens: Vec<Token>,

    eof_token: Token,

    current_pos: usize,
}

#[derive(Debug)]
pub struct ParserResult {
    /// The ranges of the parsed statements
    pub ranges: Vec<TextRange>,
    /// The syntax errors accumulated during parsing
    pub errors: Vec<SplitDiagnostic>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let eof_token = Token::eof(usize::from(
            tokens
                .last()
                .map(|t| t.span.end())
                .unwrap_or(TextSize::from(0)),
        ));

        // Place `current_pos` on the first relevant token
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

    pub fn finish(self) -> ParserResult {
        ParserResult {
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

    pub fn start_stmt(&mut self) {
        assert!(
            self.current_stmt_start.is_none(),
            "cannot start statement within statement at {:?}",
            self.tokens.get(self.current_stmt_start.unwrap())
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
            "Must close the statement on a token that's later than the start token."
        );

        let (end_token_pos, _) = self.find_last_relevant().unwrap();

        self.stmt_ranges.push((start_token_pos, end_token_pos));

        self.current_stmt_start = None;
    }

    fn current(&self) -> &Token {
        match self.tokens.get(self.current_pos) {
            Some(token) => token,
            None => &self.eof_token,
        }
    }

    /// Advances the parser to the next relevant token and returns it.
    ///
    /// NOTE: This will skip irrelevant tokens.
    fn advance(&mut self) -> &Token {
        // can't reuse any `find_next_relevant` logic because of Mr. Borrow Checker
        let (pos, token) = self
            .tokens
            .iter()
            .enumerate()
            .skip(self.current_pos + 1)
            .find(|(_, t)| is_relevant(t))
            .unwrap_or((self.tokens.len(), &self.eof_token));

        self.current_pos = pos;
        token
    }

    fn look_ahead(&self) -> Option<&Token> {
        self.tokens
            .iter()
            .skip(self.current_pos + 1)
            .find(|t| is_relevant(t))
    }

    /// Returns `None` if there are no previous relevant tokens
    fn look_back(&self) -> Option<&Token> {
        self.find_last_relevant().map(|it| it.1)
    }

    /// Will advance if the `kind` matches the current token.
    /// Otherwise, will add a diagnostic to the internal `errors`.
    pub fn expect(&mut self, kind: SyntaxKind) {
        if self.current().kind == kind {
            self.advance();
        } else {
            self.errors.push(SplitDiagnostic::new(
                format!("Expected {:#?}", kind),
                self.current().span,
            ));
        }
    }

    fn find_last_relevant(&self) -> Option<(usize, &Token)> {
        self.tokens
            .iter()
            .enumerate()
            .take(self.current_pos)
            .rfind(|(_, t)| is_relevant(t))
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

#[cfg(test)]
mod tests {
    use pgt_lexer::SyntaxKind;

    use crate::parser::Parser;

    #[test]
    fn advance_works_as_expected() {
        let sql = r#"
        create table users (
            id serial primary key,
            name text,
            email text
        );
        "#;
        let tokens = pgt_lexer::lex(sql).unwrap();
        let total_num_tokens = tokens.len();

        let mut parser = Parser::new(tokens);

        let expected = vec![
            (SyntaxKind::Create, 2),
            (SyntaxKind::Table, 4),
            (SyntaxKind::Ident, 6),
            (SyntaxKind::Ascii40, 8),
            (SyntaxKind::Ident, 11),
            (SyntaxKind::Ident, 13),
            (SyntaxKind::Primary, 15),
            (SyntaxKind::Key, 17),
            (SyntaxKind::Ascii44, 18),
            (SyntaxKind::NameP, 21),
            (SyntaxKind::TextP, 23),
            (SyntaxKind::Ascii44, 24),
            (SyntaxKind::Ident, 27),
            (SyntaxKind::TextP, 29),
            (SyntaxKind::Ascii41, 32),
            (SyntaxKind::Ascii59, 33),
        ];

        for (kind, pos) in expected {
            assert_eq!(parser.current().kind, kind);
            assert_eq!(parser.current_pos, pos);
            parser.advance();
        }

        assert_eq!(parser.current().kind, SyntaxKind::Eof);
        assert_eq!(parser.current_pos, total_num_tokens);
    }
}
