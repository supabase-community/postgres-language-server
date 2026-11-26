use crate::class::CharClass;
use crate::cursor::Cursor;
use crate::{LiteralKind, NamedParamKind, TokenKind};

/// A quoted string or identifier, by the start condition of Postgres' scanner that reads it.
///
/// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L178-L188
#[derive(Clone, Copy)]
pub(crate) enum Quote {
    /// `'…'`
    Xq,
    /// `E'…'`
    Xe,
    /// `B'…'`
    Xb,
    /// `X'…'`
    Xh,
    /// `U&'…'`
    Xus,
    /// `U&"…"`
    Xui,
    /// `"…"`
    Xd,
}

impl Quote {
    /// The prefixed string or identifier that starts `text`: `E'`, `B'`, `X'`, `U&'` or `U&"`.
    /// Postgres takes the longest match, so `e'a'` is a string, not `e` and a string, and `U&x`
    /// is `U`, `&` and `x`:
    /// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L250-L307
    pub(crate) fn prefixed(text: &str) -> Option<Quote> {
        match text.as_bytes() {
            [b'e' | b'E', b'\'', ..] => Some(Quote::Xe),
            [b'b' | b'B', b'\'', ..] => Some(Quote::Xb),
            [b'x' | b'X', b'\'', ..] => Some(Quote::Xh),
            [b'u' | b'U', b'&', b'\'', ..] => Some(Quote::Xus),
            [b'u' | b'U', b'&', b'"', ..] => Some(Quote::Xui),
            _ => None,
        }
    }

    /// The length of the start, with its opening quote.
    fn start_len(self) -> usize {
        match self {
            Quote::Xq | Quote::Xd => 1,
            Quote::Xe | Quote::Xb | Quote::Xh => 2,
            Quote::Xus | Quote::Xui => 3,
        }
    }

    fn delimiter(self) -> char {
        match self {
            Quote::Xui | Quote::Xd => '"',
            _ => '\'',
        }
    }

    /// Whether two quotes are a quote in it. In a bit or hex string, `''` ends the string and
    /// starts an empty one:
    /// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L633
    /// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L819
    fn doubles_quote(self) -> bool {
        !matches!(self, Quote::Xb | Quote::Xh)
    }

    /// Whether a backslash escapes the next character, so `E'\''` is one quote:
    /// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L706
    fn backslash_escapes(self) -> bool {
        matches!(self, Quote::Xe)
    }

    /// Whether a string on a later line continues it, after `xqs` looked for `quotecontinue`:
    /// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L574-L593
    fn continues(self) -> bool {
        !matches!(self, Quote::Xui | Quote::Xd)
    }

    fn kind(self, terminated: bool) -> TokenKind {
        let kind = match self {
            Quote::Xq => LiteralKind::Str { terminated },
            Quote::Xe => LiteralKind::EscStr { terminated },
            Quote::Xb => LiteralKind::BitStr { terminated },
            Quote::Xh => LiteralKind::ByteStr { terminated },
            Quote::Xus => LiteralKind::UnicodeEscStr { terminated },
            Quote::Xui | Quote::Xd => return TokenKind::QuotedIdent { terminated },
        };
        TokenKind::Literal { kind }
    }
}

impl Cursor<'_> {
    /// Eats a quoted string or identifier after the first character of its start.
    pub(crate) fn quoted(&mut self, quote: Quote) -> TokenKind {
        for _ in 1..quote.start_len() {
            self.bump();
        }
        quote.kind(self.quoted_text(quote))
    }

    /// Eats a quoted text after its opening quote, with the strings that continue it, and
    /// returns true if it's terminated.
    pub(crate) fn quoted_text(&mut self, quote: Quote) -> bool {
        loop {
            if !self.quoted_part(quote) {
                return false;
            }
            if !quote.continues() || !self.eat_quote_continuation() {
                return true;
            }
        }
    }

    /// Eats a quoted text after its opening quote, up to its closing quote, and returns true if
    /// it's terminated.
    fn quoted_part(&mut self, quote: Quote) -> bool {
        let delimiter = quote.delimiter();
        while let Some(c) = self.bump() {
            if c == delimiter {
                if !quote.doubles_quote() || self.first() != delimiter {
                    return true;
                }
                self.bump();
            } else if c == '\\' && quote.backslash_escapes() {
                self.bump();
            }
        }
        false
    }

    /// Eats what continues a string constant after its closing quote, up to and including the
    /// next string's opening quote: whitespace with at least one line break, and comments. Eats
    /// nothing and returns false if no string continues it.
    ///
    /// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L224-L230
    /// special_whitespace ({space}+|{comment}{newline})
    /// non_newline_whitespace ({non_newline_space}|{comment})
    /// whitespace_with_newline ({non_newline_whitespace}*{newline}{special_whitespace}*)
    /// quotecontinue {whitespace_with_newline}{quote}
    ///
    /// A comment runs to a line break or the end of the input, so it's always followed by a
    /// line break where a quote can follow at all.
    pub(crate) fn eat_quote_continuation(&mut self) -> bool {
        let mut ahead = self.clone();
        let mut line_break = false;
        loop {
            match ahead.first() {
                c if CharClass::Newline.contains(c) => {
                    ahead.bump();
                    line_break = true;
                }
                c if CharClass::NonNewlineSpace.contains(c) => {
                    ahead.bump();
                }
                '-' if ahead.second() == '-' => {
                    ahead.bump();
                    ahead.line_comment();
                }
                '\'' if line_break => {
                    ahead.bump();
                    *self = ahead;
                    return true;
                }
                _ => return false,
            }
        }
    }

    /// Whether an identifier starts here, and no prefixed string or identifier, which Postgres
    /// reads instead as the longer match. So `:e'a'`, `@b'1'` and `$U&'a'` are an operator or
    /// `$` and a string, not a named parameter.
    pub(crate) fn at_identifier(&self) -> bool {
        CharClass::IdentStart.contains(self.first()) && Quote::prefixed(self.as_str()).is_none()
    }

    /// Eats an identifier after its first character. One that starts with the letter of a
    /// prefix (`E`, `B`, `X` or `U`) and runs into a quote or `#` is an unknown prefix.
    pub(crate) fn ident(&mut self, first_char: char) -> TokenKind {
        self.eat_while(|c| CharClass::IdentCont.contains(c));
        if matches!(first_char, 'e' | 'E' | 'b' | 'B' | 'x' | 'X' | 'u' | 'U')
            && matches!(self.first(), '#' | '"' | '\'')
        {
            TokenKind::UnknownPrefix
        } else {
            TokenKind::Ident
        }
    }

    /// Eats what follows a `$`: a dollar-quoted string, a positional parameter like `$1`, or a
    /// named parameter like `$name` if no `$` ends the tag.
    ///
    /// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L285-L287
    /// `dolq_start [A-Za-z\200-\377_]`
    /// `dolq_cont [A-Za-z\200-\377_0-9]`
    /// `dolqdelim \$({dolq_start}{dolq_cont}*)?\$`
    /// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L402
    /// `param \${decdigit}+`
    pub(crate) fn dollar(&mut self) -> TokenKind {
        if !self.at_identifier() && self.first() != '$' {
            self.eat_while(|c| c.is_ascii_digit());
            return TokenKind::PositionalParam;
        }
        let rest = self.as_str();
        self.eat_while(|c| c != '$' && CharClass::IdentCont.contains(c));
        let tag = self.eaten_since(rest);
        if self.first() != '$' {
            return TokenKind::NamedParam {
                kind: NamedParamKind::DollarRaw,
            };
        }
        self.bump();
        self.dollar_quoted_string(tag)
    }

    /// Eats a dollar-quoted string after its start.
    ///
    /// The string ends only at the full `$tag$` of its start, as in Postgres' `<xdolq>{dolqdelim}`
    /// rule:
    /// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L757
    /// A `$` that doesn't start it is part of the string, so `myfn$` or `$other$` inside
    /// `$fn$ … $fn$` doesn't end it. Tags can't contain `$`, so checking each `$` is the same as
    /// Postgres' longest match.
    ///
    /// https://www.postgresql.org/docs/current/sql-syntax-lexical.html#SQL-SYNTAX-DOLLAR-QUOTING
    fn dollar_quoted_string(&mut self, tag: &str) -> TokenKind {
        let terminated = loop {
            self.eat_while(|c| c != '$');
            if self.bump().is_none() {
                break false;
            }
            if self
                .as_str()
                .strip_prefix(tag)
                .is_some_and(|after| after.starts_with('$'))
            {
                self.bump_bytes(tag.len() + 1);
                break true;
            }
        };
        TokenKind::Literal {
            kind: LiteralKind::DollarQuotedString { terminated },
        }
    }
}
