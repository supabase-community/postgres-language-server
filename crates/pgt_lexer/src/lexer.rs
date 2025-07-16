use pgt_tokenizer::tokenize;

use crate::SyntaxKind;
use crate::lexed::{LexError, Lexed};

/// Lexer that processes input text into tokens and diagnostics
pub struct Lexer<'a> {
    text: &'a str,
    kind: Vec<SyntaxKind>,
    start: Vec<u32>,
    error: Vec<LexError>,
    offset: usize,
    /// we store line ending counts outside of SyntaxKind because of the u16 represenation of SyntaxKind
    line_ending_counts: Vec<usize>,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given text
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            kind: Vec::new(),
            start: Vec::new(),
            error: Vec::new(),
            offset: 0,
            line_ending_counts: Vec::new(),
        }
    }

    /// Lex the input text and return the result
    pub fn lex(mut self) -> Lexed<'a> {
        for token in tokenize(&self.text[self.offset..]) {
            let token_text = &self.text[self.offset..][..token.len as usize];
            self.extend_token(&token.kind, token_text);
        }

        // Add EOF token
        self.push(SyntaxKind::EOF, 0, None, None);

        Lexed {
            text: self.text,
            kind: self.kind,
            start: self.start,
            error: self.error,
            line_ending_counts: self.line_ending_counts,
        }
    }

    fn push(
        &mut self,
        kind: SyntaxKind,
        len: usize,
        err: Option<&str>,
        line_ending_count: Option<usize>,
    ) {
        self.kind.push(kind);
        self.start.push(self.offset as u32);
        self.offset += len;

        assert!(
            kind != SyntaxKind::LINE_ENDING || line_ending_count.is_some(),
            "Line ending token must have a line ending count"
        );

        self.line_ending_counts.push(line_ending_count.unwrap_or(0));

        if let Some(err) = err {
            let token = (self.kind.len() - 1) as u32;
            let msg = err.to_owned();
            self.error.push(LexError { msg, token });
        }
    }

    fn extend_token(&mut self, kind: &pgt_tokenizer::TokenKind, token_text: &str) {
        let mut err = "";
        let mut line_ending_count = None;

        let syntax_kind = {
            match kind {
                pgt_tokenizer::TokenKind::LineComment => SyntaxKind::COMMENT,
                pgt_tokenizer::TokenKind::BlockComment { terminated } => {
                    if !terminated {
                        err = "Missing trailing `*/` symbols to terminate the block comment";
                    }
                    SyntaxKind::COMMENT
                }
                pgt_tokenizer::TokenKind::Space => SyntaxKind::SPACE,
                pgt_tokenizer::TokenKind::Tab => SyntaxKind::TAB,
                pgt_tokenizer::TokenKind::LineEnding { count } => {
                    line_ending_count = Some(*count);
                    SyntaxKind::LINE_ENDING
                }
                pgt_tokenizer::TokenKind::VerticalTab => SyntaxKind::VERTICAL_TAB,
                pgt_tokenizer::TokenKind::FormFeed => SyntaxKind::FORM_FEED,
                pgt_tokenizer::TokenKind::Ident => {
                    SyntaxKind::from_keyword(token_text).unwrap_or(SyntaxKind::IDENT)
                }
                pgt_tokenizer::TokenKind::Literal { kind, .. } => {
                    self.extend_literal(token_text.len(), kind);
                    return;
                }
                pgt_tokenizer::TokenKind::Semi => SyntaxKind::SEMICOLON,
                pgt_tokenizer::TokenKind::Comma => SyntaxKind::COMMA,
                pgt_tokenizer::TokenKind::Dot => SyntaxKind::DOT,
                pgt_tokenizer::TokenKind::OpenParen => SyntaxKind::L_PAREN,
                pgt_tokenizer::TokenKind::CloseParen => SyntaxKind::R_PAREN,
                pgt_tokenizer::TokenKind::OpenBracket => SyntaxKind::L_BRACK,
                pgt_tokenizer::TokenKind::CloseBracket => SyntaxKind::R_BRACK,
                pgt_tokenizer::TokenKind::At => SyntaxKind::AT,
                pgt_tokenizer::TokenKind::Pound => SyntaxKind::POUND,
                pgt_tokenizer::TokenKind::Tilde => SyntaxKind::TILDE,
                pgt_tokenizer::TokenKind::Question => SyntaxKind::QUESTION,
                pgt_tokenizer::TokenKind::Colon => SyntaxKind::COLON,
                pgt_tokenizer::TokenKind::Eq => SyntaxKind::EQ,
                pgt_tokenizer::TokenKind::Bang => SyntaxKind::BANG,
                pgt_tokenizer::TokenKind::Lt => SyntaxKind::L_ANGLE,
                pgt_tokenizer::TokenKind::Gt => SyntaxKind::R_ANGLE,
                pgt_tokenizer::TokenKind::Minus => SyntaxKind::MINUS,
                pgt_tokenizer::TokenKind::And => SyntaxKind::AMP,
                pgt_tokenizer::TokenKind::Or => SyntaxKind::PIPE,
                pgt_tokenizer::TokenKind::Plus => SyntaxKind::PLUS,
                pgt_tokenizer::TokenKind::Star => SyntaxKind::STAR,
                pgt_tokenizer::TokenKind::Slash => SyntaxKind::SLASH,
                pgt_tokenizer::TokenKind::Caret => SyntaxKind::CARET,
                pgt_tokenizer::TokenKind::Percent => SyntaxKind::PERCENT,
                pgt_tokenizer::TokenKind::Unknown => SyntaxKind::ERROR,
                pgt_tokenizer::TokenKind::Backslash => SyntaxKind::BACKSLASH,
                pgt_tokenizer::TokenKind::UnknownPrefix => {
                    err = "unknown literal prefix";
                    SyntaxKind::IDENT
                }
                pgt_tokenizer::TokenKind::Eof => SyntaxKind::EOF,
                pgt_tokenizer::TokenKind::Backtick => SyntaxKind::BACKTICK,
                pgt_tokenizer::TokenKind::PositionalParam => SyntaxKind::POSITIONAL_PARAM,
                pgt_tokenizer::TokenKind::NamedParam { kind } => {
                    match kind {
                        pgt_tokenizer::NamedParamKind::ColonIdentifier { terminated: false } => {
                            err = "Missing trailing \" to terminate the named parameter";
                        }
                        pgt_tokenizer::NamedParamKind::ColonString { terminated: false } => {
                            err = "Missing trailing ' to terminate the named parameter";
                        }
                        _ => {}
                    };
                    SyntaxKind::POSITIONAL_PARAM
                }
                pgt_tokenizer::TokenKind::QuotedIdent { terminated } => {
                    if !terminated {
                        err = "Missing trailing \" to terminate the quoted identifier"
                    }
                    SyntaxKind::IDENT
                }
            }
        };

        let err = if err.is_empty() { None } else { Some(err) };
        self.push(syntax_kind, token_text.len(), err, line_ending_count);
    }

    fn extend_literal(&mut self, len: usize, kind: &pgt_tokenizer::LiteralKind) {
        let mut err = "";

        let syntax_kind = match *kind {
            pgt_tokenizer::LiteralKind::Int { empty_int, base: _ } => {
                if empty_int {
                    err = "Missing digits after the integer base prefix";
                }
                SyntaxKind::INT_NUMBER
            }
            pgt_tokenizer::LiteralKind::Float {
                empty_exponent,
                base: _,
            } => {
                if empty_exponent {
                    err = "Missing digits after the exponent symbol";
                }
                SyntaxKind::FLOAT_NUMBER
            }
            pgt_tokenizer::LiteralKind::Str { terminated } => {
                if !terminated {
                    err = "Missing trailing `'` symbol to terminate the string literal";
                }
                SyntaxKind::STRING
            }
            pgt_tokenizer::LiteralKind::ByteStr { terminated } => {
                if !terminated {
                    err = "Missing trailing `'` symbol to terminate the hex bit string literal";
                }
                SyntaxKind::BYTE_STRING
            }
            pgt_tokenizer::LiteralKind::BitStr { terminated } => {
                if !terminated {
                    err = "Missing trailing `'` symbol to terminate the bit string literal";
                }
                SyntaxKind::BIT_STRING
            }
            pgt_tokenizer::LiteralKind::DollarQuotedString { terminated } => {
                if !terminated {
                    err = "Unterminated dollar quoted string literal";
                }
                SyntaxKind::DOLLAR_QUOTED_STRING
            }
            pgt_tokenizer::LiteralKind::UnicodeEscStr { terminated } => {
                if !terminated {
                    err = "Missing trailing `'` symbol to terminate the unicode escape string literal";
                }
                SyntaxKind::BYTE_STRING
            }
            pgt_tokenizer::LiteralKind::EscStr { terminated } => {
                if !terminated {
                    err = "Missing trailing `'` symbol to terminate the escape string literal";
                }
                SyntaxKind::ESC_STRING
            }
        };

        let err = if err.is_empty() { None } else { Some(err) };
        self.push(syntax_kind, len, err, None);
    }
}
