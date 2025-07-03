// based on https://github.com/rust-lang/rust-analyzer/blob/d8887c0758bbd2d5f752d5bd405d4491e90e7ed6/crates/parser/src/lexed_str.rs

use std::ops;

use pgt_tokenizer::tokenize;

use crate::SyntaxKind;

pub struct LexedStr<'a> {
    text: &'a str,
    kind: Vec<SyntaxKind>,
    start: Vec<u32>,
    error: Vec<LexError>,
}

struct LexError {
    msg: String,
    token: u32,
}

impl<'a> LexedStr<'a> {
    pub fn new(text: &'a str) -> LexedStr<'a> {
        let mut conv = Converter::new(text);

        for token in tokenize(&text[conv.offset..]) {
            let token_text = &text[conv.offset..][..token.len as usize];

            conv.extend_token(&token.kind, token_text);
        }

        conv.finalize_with_eof()
    }

    pub fn len(&self) -> usize {
        self.kind.len() - 1
    }

    pub fn kind(&self, i: usize) -> SyntaxKind {
        assert!(i < self.len());
        self.kind[i]
    }

    pub fn text(&self, i: usize) -> &str {
        self.range_text(i..i + 1)
    }

    pub fn range_text(&self, r: ops::Range<usize>) -> &str {
        assert!(r.start < r.end && r.end <= self.len());
        let lo = self.start[r.start] as usize;
        let hi = self.start[r.end] as usize;
        &self.text[lo..hi]
    }

    // Naming is hard.
    pub fn text_range(&self, i: usize) -> ops::Range<usize> {
        assert!(i < self.len());
        let lo = self.start[i] as usize;
        let hi = self.start[i + 1] as usize;
        lo..hi
    }

    pub fn errors(&self) -> impl Iterator<Item = (usize, &str)> + '_ {
        self.error
            .iter()
            .map(|it| (it.token as usize, it.msg.as_str()))
    }

    fn push(&mut self, kind: SyntaxKind, offset: usize) {
        self.kind.push(kind);
        self.start.push(offset as u32);
    }
}

struct Converter<'a> {
    res: LexedStr<'a>,
    offset: usize,
}

impl<'a> Converter<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            res: LexedStr {
                text,
                kind: Vec::new(),
                start: Vec::new(),
                error: Vec::new(),
            },
            offset: 0,
        }
    }

    fn finalize_with_eof(mut self) -> LexedStr<'a> {
        self.res.push(SyntaxKind::EOF, self.offset);
        self.res
    }

    fn push(&mut self, kind: SyntaxKind, len: usize, err: Option<&str>) {
        self.res.push(kind, self.offset);
        self.offset += len;

        if let Some(err) = err {
            let token = self.res.len() as u32;
            let msg = err.to_owned();
            self.res.error.push(LexError { msg, token });
        }
    }

    fn extend_token(&mut self, kind: &pgt_tokenizer::TokenKind, token_text: &str) {
        // A note on an intended tradeoff:
        // We drop some useful information here (see patterns with double dots `..`)
        // Storing that info in `SyntaxKind` is not possible due to its layout requirements of
        // being `u16` that come from `rowan::SyntaxKind`.
        let mut err = "";

        let syntax_kind = {
            match kind {
                pgt_tokenizer::TokenKind::LineComment => SyntaxKind::COMMENT,
                pgt_tokenizer::TokenKind::BlockComment { terminated } => {
                    if !terminated {
                        err = "Missing trailing `*/` symbols to terminate the block comment";
                    }
                    SyntaxKind::COMMENT
                }

                // whitespace
                pgt_tokenizer::TokenKind::Space => SyntaxKind::SPACE,
                pgt_tokenizer::TokenKind::Tab => SyntaxKind::TAB,
                pgt_tokenizer::TokenKind::Newline => SyntaxKind::NEWLINE,
                pgt_tokenizer::TokenKind::CarriageReturn => SyntaxKind::CARRIAGE_RETURN,
                pgt_tokenizer::TokenKind::VerticalTab => SyntaxKind::VERTICAL_TAB,
                pgt_tokenizer::TokenKind::FormFeed => SyntaxKind::FORM_FEED,
                pgt_tokenizer::TokenKind::Ident => {
                    // TODO: check for max identifier length
                    //
                    // see: https://www.postgresql.org/docs/16/sql-syntax-lexical.html#SQL-SYNTAX-IDENTIFIERS
                    // The system uses no more than NAMEDATALEN-1 bytes of an
                    // identifier; longer names can be written in commands, but
                    // they will be truncated. By default, NAMEDATALEN is 64 so
                    // the maximum identifier length is 63 bytes. If this limit
                    // is problematic, it can be raised by changing the
                    // NAMEDATALEN constant in src/include/pg_config_manual.h.
                    // see: https://github.com/postgres/postgres/blob/e032e4c7ddd0e1f7865b246ec18944365d4f8614/src/include/pg_config_manual.h#L29
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
                pgt_tokenizer::TokenKind::UnknownPrefix => {
                    err = "unknown literal prefix";
                    SyntaxKind::IDENT
                }
                pgt_tokenizer::TokenKind::Eof => SyntaxKind::EOF,
                pgt_tokenizer::TokenKind::Backtick => SyntaxKind::BACKTICK,
                pgt_tokenizer::TokenKind::PositionalParam => SyntaxKind::POSITIONAL_PARAM,
                pgt_tokenizer::TokenKind::QuotedIdent { terminated } => {
                    if !terminated {
                        err = "Missing trailing \" to terminate the quoted identifier"
                    }
                    SyntaxKind::IDENT
                }
            }
        };

        let err = if err.is_empty() { None } else { Some(err) };
        self.push(syntax_kind, token_text.len(), err);
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
                // TODO: rust analzyer checks for un-escaped strings, we should too
                SyntaxKind::STRING
            }
            pgt_tokenizer::LiteralKind::ByteStr { terminated } => {
                if !terminated {
                    err = "Missing trailing `'` symbol to terminate the hex bit string literal";
                }
                // TODO: rust analzyer checks for un-escaped strings, we should too
                SyntaxKind::BYTE_STRING
            }
            pgt_tokenizer::LiteralKind::BitStr { terminated } => {
                if !terminated {
                    err = "Missing trailing `\'` symbol to terminate the bit string literal";
                }
                // TODO: rust analzyer checks for un-escaped strings, we should too
                SyntaxKind::BIT_STRING
            }
            pgt_tokenizer::LiteralKind::DollarQuotedString { terminated } => {
                if !terminated {
                    // TODO: we could be fancier and say the ending string we're looking for
                    err = "Unterminated dollar quoted string literal";
                }
                // TODO: rust analzyer checks for un-escaped strings, we should too
                SyntaxKind::DOLLAR_QUOTED_STRING
            }
            pgt_tokenizer::LiteralKind::UnicodeEscStr { terminated } => {
                if !terminated {
                    err = "Missing trailing `'` symbol to terminate the unicode escape string literal";
                }
                // TODO: rust analzyer checks for un-escaped strings, we should too
                SyntaxKind::BYTE_STRING
            }
            pgt_tokenizer::LiteralKind::EscStr { terminated } => {
                if !terminated {
                    err = "Missing trailing `\'` symbol to terminate the escape string literal";
                }
                // TODO: rust analzyer checks for un-escaped strings, we should too
                SyntaxKind::ESC_STRING
            }
        };

        let err = if err.is_empty() { None } else { Some(err) };
        self.push(syntax_kind, len, err);
    }
}
