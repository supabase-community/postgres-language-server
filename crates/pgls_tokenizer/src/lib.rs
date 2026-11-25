mod cursor;
mod token;
use cursor::{Cursor, EOF_CHAR};
pub use token::{Base, LiteralKind, NamedParamKind, Token, TokenKind};

// via: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L332-L333
// ident_start		[A-Za-z\200-\377_]
// ident_cont		[A-Za-z\200-\377_0-9\$]
//
// `\200-\377` are bytes, so every non-ASCII character is part of an identifier: all bytes of
// its UTF-8 encoding are in that range.
const fn is_ident_start(c: char) -> bool {
    matches!(c, 'a'..='z' | 'A'..='Z' | '_') || !c.is_ascii()
}

const fn is_ident_cont(c: char) -> bool {
    matches!(c, 'a'..='z' | 'A'..='Z' | '_' | '0'..='9' | '$') || !c.is_ascii()
}

// whitespace
// - https://github.com/postgres/postgres/blob/db0c96cc18aec417101e37e59fcc53d4bf647915/src/backend/parser/scansup.c#L107-L128
// - https://github.com/postgres/postgres/blob/db0c96cc18aec417101e37e59fcc53d4bf647915/src/backend/parser/scan.l#L204-L229

const fn is_space(c: char) -> bool {
    matches!(
        c, ' ' // space
    )
}

const fn is_tab(c: char) -> bool {
    matches!(
        c, '\t' // tab
    )
}

const fn is_line_ending(c: char) -> bool {
    matches!(
        c,
        '\n' | '\r' // newline or carriage return
    )
}

const fn is_vertical_tab(c: char) -> bool {
    matches!(
        c, '\u{000B}' // vertical tab
    )
}

const fn is_form_feed(c: char) -> bool {
    matches!(
        c, '\u{000C}' // form feed
    )
}

impl Cursor<'_> {
    // see: https://github.com/rust-lang/rust/blob/ba1d7f4a083e6402679105115ded645512a7aea8/compiler/rustc_lexer/src/lib.rs#L339
    pub(crate) fn advance_token(&mut self) -> Token {
        let Some(first_char) = self.bump() else {
            return Token::new(TokenKind::Eof, 0);
        };
        let token_kind = match first_char {
            // Slash, comment or block comment.
            '/' => match self.first() {
                '*' => self.block_comment(),
                _ => TokenKind::Slash,
            },
            '-' => match self.first() {
                '-' => self.line_comment(),
                _ => TokenKind::Minus,
            },

            c if is_space(c) => {
                self.eat_while(is_space);
                TokenKind::Space
            }

            c if is_tab(c) => {
                self.eat_while(is_tab);
                TokenKind::Tab
            }

            c if is_line_ending(c) => self.line_ending_sequence(c),

            c if is_vertical_tab(c) => {
                self.eat_while(is_vertical_tab);
                TokenKind::VerticalTab
            }

            c if is_form_feed(c) => {
                self.eat_while(is_form_feed);
                TokenKind::FormFeed
            }

            // https://www.postgresql.org/docs/current/sql-syntax-lexical.html#SQL-SYNTAX-STRINGS-UESCAPE
            //
            // Only `U&'` and `U&"` start one, so `U&x` is `U`, `&` and `x`:
            // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L301-L304
            'u' | 'U' => match (self.first(), self.second()) {
                ('&', '\'' | '"') => {
                    self.bump();
                    self.prefixed_string(
                        |terminated| LiteralKind::UnicodeEscStr { terminated },
                        true,
                    )
                }
                _ => self.ident_or_unknown_prefix(),
            },

            // escaped strings
            'e' | 'E' => {
                self.prefixed_string(|terminated| LiteralKind::EscStr { terminated }, false)
            }

            // bit string
            'b' | 'B' => {
                self.prefixed_string(|terminated| LiteralKind::BitStr { terminated }, false)
            }

            // hexadecimal byte string
            'x' | 'X' => {
                self.prefixed_string(|terminated| LiteralKind::ByteStr { terminated }, false)
            }

            // Identifier (this should be checked after other variant that can
            // start as identifier).
            c if is_ident_start(c) => self.ident(),

            // Numeric literal.
            // see: https://www.postgresql.org/docs/current/sql-syntax-lexical.html#SQL-SYNTAX-CONSTANTS-NUMERIC
            c @ '0'..='9' => {
                let literal_kind = self.number(c);
                TokenKind::Literal { kind: literal_kind }
            }
            '.' => match self.first() {
                '0'..='9' => TokenKind::Literal {
                    kind: self.fraction(),
                },
                // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L339
                // dot_dot			\.\.
                '.' => {
                    self.bump();
                    TokenKind::DotDot
                }
                _ => TokenKind::Dot,
            },
            '@' => {
                if is_ident_start(self.first()) && !self.at_prefixed_string() {
                    // Named parameter with @ prefix.
                    self.eat_while(is_ident_cont);
                    TokenKind::NamedParam {
                        kind: NamedParamKind::AtPrefix,
                    }
                } else {
                    TokenKind::At
                }
            }
            ':' => {
                if self.first() == ':' {
                    self.bump();
                    TokenKind::DoubleColon
                } else {
                    // Named parameters in psql with different substitution styles.
                    //
                    // https://www.postgresql.org/docs/current/app-psql.html#APP-PSQL-INTERPOLATION
                    match self.first() {
                        '\'' => {
                            // Named parameter with colon prefix and single quotes.
                            self.bump();
                            let terminated = self.single_quoted_string(true, false);
                            let kind = NamedParamKind::ColonString { terminated };
                            TokenKind::NamedParam { kind }
                        }
                        '"' => {
                            // Named parameter with colon prefix and double quotes.
                            self.bump();
                            let terminated = self.double_quoted_string();
                            let kind = NamedParamKind::ColonIdentifier { terminated };
                            TokenKind::NamedParam { kind }
                        }
                        c if is_ident_start(c) && !self.at_prefixed_string() => {
                            // Named parameter with colon prefix.
                            self.eat_while(is_ident_cont);
                            TokenKind::NamedParam {
                                kind: NamedParamKind::ColonRaw,
                            }
                        }
                        _ => TokenKind::Colon,
                    }
                }
            }
            // One-symbol tokens.
            ';' => TokenKind::Semi,
            '\\' => TokenKind::Backslash,
            ',' => TokenKind::Comma,
            '(' => TokenKind::OpenParen,
            ')' => TokenKind::CloseParen,
            '[' => TokenKind::OpenBracket,
            ']' => TokenKind::CloseBracket,
            '#' => TokenKind::Pound,
            '~' => TokenKind::Tilde,
            '?' => TokenKind::Question,
            '$' => {
                // Dollar quoted strings
                if (is_ident_start(self.first()) && !self.at_prefixed_string())
                    || self.first() == '$'
                {
                    // Get the start sequence of the dollar quote, i.e., 'foo' in $foo$hello$foo$
                    // if ident does not continue and there is no terminating dollar
                    // sign, we have a positional param `$name`
                    let mut start = vec![];
                    loop {
                        match self.first() {
                            '$' => {
                                self.bump();
                                break self.dollar_quoted_string(start);
                            }
                            c if is_ident_cont(c) => {
                                self.bump();
                                start.push(c);
                            }
                            _ => {
                                break TokenKind::NamedParam {
                                    kind: NamedParamKind::DollarRaw,
                                };
                            }
                        }
                    }
                } else {
                    // positional parameter, e.g. `$1`
                    while self.first().is_ascii_digit() {
                        self.bump();
                    }
                    TokenKind::PositionalParam
                }
            }
            '`' => TokenKind::Backtick,
            '=' => TokenKind::Eq,
            '!' => TokenKind::Bang,
            '<' => TokenKind::Lt,
            '>' => TokenKind::Gt,
            '&' => TokenKind::And,
            '|' => TokenKind::Or,
            '+' => TokenKind::Plus,
            '*' => TokenKind::Star,
            '^' => TokenKind::Caret,
            '%' => TokenKind::Percent,

            // String literal
            '\'' => self.string_constant(|terminated| LiteralKind::Str { terminated }),

            // Quoted indentifiers
            '"' => {
                let terminated = self.double_quoted_string();
                TokenKind::QuotedIdent { terminated }
            }
            _ => TokenKind::Unknown,
        };
        let res = Token::new(token_kind, self.pos_within_token());
        self.reset_pos_within_token();
        res
    }
    pub(crate) fn ident(&mut self) -> TokenKind {
        self.eat_while(is_ident_cont);
        TokenKind::Ident
    }

    fn ident_or_unknown_prefix(&mut self) -> TokenKind {
        // Start is already eaten, eat the rest of identifier.
        self.eat_while(is_ident_cont);
        // Known prefixes must have been handled earlier. So if
        // we see a prefix here, it is definitely an unknown prefix.
        match self.first() {
            '#' | '"' | '\'' => TokenKind::UnknownPrefix,
            _ => TokenKind::Ident,
        }
    }

    // see: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L211-L213
    // non_newline		[^\n\r]
    // comment			("--"{non_newline}*)
    pub(crate) fn line_comment(&mut self) -> TokenKind {
        self.bump();

        self.eat_while(|c| !is_line_ending(c));
        TokenKind::LineComment
    }

    // see: https://github.com/postgres/postgres/blob/db0c96cc18aec417101e37e59fcc53d4bf647915/src/backend/parser/scan.l#L324-L344
    pub(crate) fn block_comment(&mut self) -> TokenKind {
        self.bump();

        let mut depth = 1usize;
        while let Some(c) = self.bump() {
            match c {
                '/' if self.first() == '*' => {
                    self.bump();
                    depth += 1;
                }
                '*' if self.first() == '/' => {
                    self.bump();
                    depth -= 1;
                    if depth == 0 {
                        // This block comment is closed, so for a construction like "/* */ */"
                        // there will be a successfully parsed block comment "/* */"
                        // and " */" will be processed separately.
                        break;
                    }
                }
                _ => (),
            }
        }

        TokenKind::BlockComment {
            terminated: depth == 0,
        }
    }

    // invariant: we care about the number of consecutive newlines so we count them.
    //
    // Postgres considers a DOS-style \r\n sequence as two successive newlines, but we care about
    // logical line breaks and consider \r\n as one logical line break
    fn line_ending_sequence(&mut self, prev: char) -> TokenKind {
        // already consumed first line ending character (\n or \r)
        let mut line_breaks = 1;

        // started with \r, check if it's part of \r\n
        if prev == '\r' && self.first() == '\n' {
            // consume the \n - \r\n still counts as 1 logical line break
            self.bump();
        }

        // continue checking for more line endings
        loop {
            match self.first() {
                '\r' if self.second() == '\n' => {
                    self.bump(); // consume \r
                    self.bump(); // consume \n
                    line_breaks += 1;
                }
                '\n' => {
                    self.bump();
                    line_breaks += 1;
                }
                '\r' => {
                    self.bump();
                    line_breaks += 1;
                }
                _ => break,
            }
        }

        TokenKind::LineEnding { count: line_breaks }
    }

    fn prefixed_string(
        &mut self,
        mk_kind: fn(bool) -> LiteralKind,
        allows_double: bool,
    ) -> TokenKind {
        match self.first() {
            '\'' => {
                self.bump();
                self.string_constant(mk_kind)
            }
            '"' if allows_double => {
                self.bump();
                let terminated = self.double_quoted_string();
                TokenKind::QuotedIdent { terminated }
            }
            _ => self.ident_or_unknown_prefix(),
        }
    }

    fn number(&mut self, first_digit: char) -> LiteralKind {
        let base = Base::Decimal;
        if first_digit == '0' {
            // Attempt to parse encoding base. A binary, octal or hexadecimal integer has no
            // fraction or exponent, so `0b1.a` is `0b1`, `.` and `a`:
            // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L387-L389
            match self.first() {
                'b' | 'B' => {
                    self.bump();
                    let empty_int = !self.eat_decimal_digits();
                    return LiteralKind::Int {
                        base: Base::Binary,
                        empty_int,
                    };
                }
                'o' | 'O' => {
                    self.bump();
                    let empty_int = !self.eat_decimal_digits();
                    return LiteralKind::Int {
                        base: Base::Octal,
                        empty_int,
                    };
                }
                'x' | 'X' => {
                    self.bump();
                    let empty_int = !self.eat_hexadecimal_digits();
                    return LiteralKind::Int {
                        base: Base::Hexadecimal,
                        empty_int,
                    };
                }
                // Not a base prefix; consume additional digits.
                '0'..='9' | '_' => {
                    self.eat_decimal_digits();
                }

                // Also not a base prefix; nothing more to do here.
                '.' | 'e' | 'E' => {}

                // Just a 0.
                _ => {
                    return LiteralKind::Int {
                        base,
                        empty_int: false,
                    };
                }
            }
        } else {
            // No base prefix, parse number in the usual way.
            self.eat_decimal_digits();
        };

        match self.first() {
            // `1..10` is `1`, `..` and `10`:
            // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L396
            // numericfail		{decinteger}\.\.
            '.' if self.second() != '.' => {
                self.bump();
                self.fraction()
            }
            'e' | 'E' => {
                self.bump();
                let empty_exponent = !self.eat_float_exponent();
                LiteralKind::Float {
                    base,
                    empty_exponent,
                }
            }
            _ => LiteralKind::Int {
                base,
                empty_int: false,
            },
        }
    }

    /// Eats the rest of a number after its `.`: the digits of the fraction, if any, and the
    /// exponent. A number has one fraction, so `.1.2` is `.1` and `.2`:
    /// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L395
    /// numeric (({decinteger}\.{decinteger}?)|(\.{decinteger}))
    fn fraction(&mut self) -> LiteralKind {
        if self.first().is_ascii_digit() {
            self.eat_decimal_digits();
        }
        let mut empty_exponent = false;
        if matches!(self.first(), 'e' | 'E') {
            self.bump();
            empty_exponent = !self.eat_float_exponent();
        }
        LiteralKind::Float {
            base: Base::Decimal,
            empty_exponent,
        }
    }

    /// Whether a prefixed string or identifier starts here: `E'`, `B'`, `X'`, `U&'` or `U&"`.
    /// Postgres takes the longest match, so `:e'a'`, `@b'1'` and `$U&'a'` are an operator or
    /// `$` and a string, not a named parameter:
    /// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L250-L304
    fn at_prefixed_string(&self) -> bool {
        let mut chars = self.as_str().chars();
        matches!(
            (chars.next(), chars.next(), chars.next()),
            (Some('e' | 'E' | 'b' | 'B' | 'x' | 'X'), Some('\''), _)
                | (Some('u' | 'U'), Some('&'), Some('\'' | '"'))
        )
    }

    /// Eats a string constant after its opening quote, with the strings that continue it.
    ///
    /// `''` is a quote in it, except in a bit or hex string, where it ends the string and starts
    /// an empty one (`xqdouble` only applies in `<xq,xe,xus>`):
    /// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L633
    ///
    /// In an escape string, a backslash escapes the next character, so `E'\''` is one quote
    /// (`xeescape` only applies in `<xe>`):
    /// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L706
    fn string_constant(&mut self, kind: fn(bool) -> LiteralKind) -> TokenKind {
        let doubled_quote = !matches!(
            kind(true),
            LiteralKind::BitStr { .. } | LiteralKind::ByteStr { .. }
        );
        let backslash_escape = matches!(kind(true), LiteralKind::EscStr { .. });
        let terminated = loop {
            if !self.single_quoted_string(doubled_quote, backslash_escape) {
                break false;
            }
            if !self.eat_quote_continuation() {
                break true;
            }
        };
        TokenKind::Literal {
            kind: kind(terminated),
        }
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
    fn eat_quote_continuation(&mut self) -> bool {
        let mut ahead = self.clone();
        let mut line_break = false;
        loop {
            match ahead.first() {
                c if is_line_ending(c) => {
                    ahead.bump();
                    line_break = true;
                }
                c if is_space(c) || is_tab(c) || is_vertical_tab(c) || is_form_feed(c) => {
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

    /// Eats a quoted text after its opening quote and returns true if it's terminated. With
    /// `doubled_quote`, `''` is a quote in it; with `backslash_escape`, a backslash escapes the
    /// next character.
    fn single_quoted_string(&mut self, doubled_quote: bool, backslash_escape: bool) -> bool {
        // Parse until either quotes are terminated or error is detected.
        loop {
            match self.first() {
                // Quotes might be terminated.
                '\'' => {
                    self.bump();

                    match self.first() {
                        // encountered an escaped quote ''
                        '\'' if doubled_quote => {
                            self.bump();
                        }
                        // encountered terminating quote
                        _ => return true,
                    }
                }
                '\\' if backslash_escape => {
                    self.bump();
                    self.bump();
                }
                // End of file, stop parsing.
                EOF_CHAR if self.is_eof() => break,
                // Skip the character.
                _ => {
                    self.bump();
                }
            }
        }
        // String was not terminated.
        false
    }

    /// Eats double-quoted string and returns true
    /// if string is terminated.
    fn double_quoted_string(&mut self) -> bool {
        while let Some(c) = self.bump() {
            match c {
                '"' if self.first() == '"' => {
                    // Bump again to skip escaped character.
                    self.bump();
                }
                '"' => {
                    return true;
                }
                _ => (),
            }
        }
        // End of file reached.
        false
    }

    // https://www.postgresql.org/docs/16/sql-syntax-lexical.html#SQL-SYNTAX-DOLLAR-QUOTING
    //
    // The string ends only at the full `$tag$` of its start, as in Postgres'
    // `<xdolq>{dolqdelim}` rule:
    // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L757
    // A `$` that doesn't start it is part of the string, so `myfn$` or `$other$`
    // inside `$fn$ … $fn$` doesn't end it. Tags can't contain `$`, so checking
    // each `$` is the same as Postgres' longest match.
    fn dollar_quoted_string(&mut self, start: Vec<char>) -> TokenKind {
        loop {
            self.eat_while(|c| c != '$');
            if self.is_eof() {
                return TokenKind::Literal {
                    kind: LiteralKind::DollarQuotedString { terminated: false },
                };
            }
            // eat $
            self.bump();

            let mut rest = self.as_str().chars();
            let closes = start.iter().all(|c| rest.next() == Some(*c)) && rest.next() == Some('$');
            if closes {
                // the tag and the closing $
                for _ in 0..=start.len() {
                    self.bump();
                }
                return TokenKind::Literal {
                    kind: LiteralKind::DollarQuotedString { terminated: true },
                };
            }
        }
    }

    fn eat_decimal_digits(&mut self) -> bool {
        let mut has_digits = false;
        loop {
            match self.first() {
                '_' => {
                    self.bump();
                }
                '0'..='9' => {
                    has_digits = true;
                    self.bump();
                }
                _ => break,
            }
        }
        has_digits
    }

    fn eat_hexadecimal_digits(&mut self) -> bool {
        let mut has_digits = false;
        loop {
            match self.first() {
                '_' => {
                    self.bump();
                }
                '0'..='9' | 'a'..='f' | 'A'..='F' => {
                    has_digits = true;
                    self.bump();
                }
                _ => break,
            }
        }
        has_digits
    }

    /// Eats the float exponent. Returns true if at least one digit was met,
    /// and returns false otherwise.
    fn eat_float_exponent(&mut self) -> bool {
        if self.first() == '-' || self.first() == '+' {
            self.bump();
        }
        self.eat_decimal_digits()
    }
}

/// Creates an iterator that produces tokens from the input string.
pub fn tokenize(input: &str) -> impl Iterator<Item = Token> + '_ {
    let mut cursor = Cursor::new(input);
    std::iter::from_fn(move || {
        let token = cursor.advance_token();
        if token.kind != TokenKind::Eof {
            Some(token)
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use std::fmt;

    use super::*;
    use insta::assert_debug_snapshot;

    struct TokenDebug<'a> {
        content: &'a str,
        token: Token,
    }
    impl fmt::Debug for TokenDebug<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{:?} @ {:?}", self.content, self.token.kind)
        }
    }

    impl<'a> TokenDebug<'a> {
        fn new(token: Token, input: &'a str, start: u32) -> TokenDebug<'a> {
            TokenDebug {
                token,
                content: &input[start as usize..(start + token.len) as usize],
            }
        }
    }

    fn lex(input: &str) -> Vec<TokenDebug<'_>> {
        let mut tokens = vec![];
        let mut start = 0;

        for token in tokenize(input) {
            let length = token.len;
            tokens.push(TokenDebug::new(token, input, start));
            start += length;
        }
        tokens
    }

    #[test]
    fn named_param_at() {
        let result = lex("select 1 from c where id = @id;");
        assert_debug_snapshot!(result);
    }

    #[test]
    fn graphile_named_param() {
        let result =
            lex("grant usage on schema public, app_public, app_hidden to :DATABASE_VISITOR;");
        assert_debug_snapshot!(result);
    }

    #[test]
    fn named_param_dollar_raw() {
        let result = lex("select 1 from c where id = $id;");
        assert_debug_snapshot!(result);
    }

    #[test]
    fn named_param_colon_raw() {
        let result = lex("select 1 from c where id = :id;");
        assert_debug_snapshot!(result);
    }

    #[test]
    fn debug_simple_cast() {
        let result = lex("::test");
        assert_debug_snapshot!(result, @r#"
        [
            "::" @ DoubleColon,
            "test" @ Ident,
        ]
        "#);
    }

    #[test]
    fn named_param_colon_raw_vs_cast() {
        let result = lex("select 1 from c where id::test = :id;");
        assert_debug_snapshot!(result);
    }

    #[test]
    fn named_param_colon_string() {
        let result = lex("select 1 from c where id = :'id';");
        assert_debug_snapshot!(result);
    }

    #[test]
    fn named_param_colon_identifier() {
        let result = lex("select 1 from c where id = :\"id\";");
        assert_debug_snapshot!(result);
    }

    #[test]
    fn lex_statement() {
        let result = lex("select 1;");
        assert_debug_snapshot!(result);
    }

    #[test]
    fn block_comment() {
        let result = lex(r#"
/*
 * foo
 * bar
*/"#);
        assert_debug_snapshot!(result);
    }

    #[test]
    fn block_comment_unterminated() {
        let result = lex(r#"
/*
 * foo
 * bar
 /*
*/"#);
        assert_debug_snapshot!(result);
    }

    #[test]
    fn line_comment() {
        let result = lex(r#"
-- foooooooooooo bar buzz
"#);
        assert_debug_snapshot!(result);
    }

    #[test]
    fn line_comment_ends_at_a_lone_carriage_return() {
        assert_debug_snapshot!(lex("select 1 -- c\rfrom t;\r\n-- d\r\n"), @r#"
        [
            "select" @ Ident,
            " " @ Space,
            "1" @ Literal { kind: Int { base: Decimal, empty_int: false } },
            " " @ Space,
            "-- c" @ LineComment,
            "\r" @ LineEnding { count: 1 },
            "from" @ Ident,
            " " @ Space,
            "t" @ Ident,
            ";" @ Semi,
            "\r\n" @ LineEnding { count: 1 },
            "-- d" @ LineComment,
            "\r\n" @ LineEnding { count: 1 },
        ]
        "#);
    }

    #[test]
    fn line_comment_whitespace() {
        assert_debug_snapshot!(lex(r#"
select 'Hello' -- This is a comment
' World';"#))
    }

    #[test]
    fn dollar_quoting() {
        assert_debug_snapshot!(lex(r#"
$$Dianne's horse$$
$SomeTag$Dianne's horse$SomeTag$

-- with dollar inside and matching tags
$foo$hello$world$bar$
"#))
    }

    #[test]
    fn dollar_strings_part2() {
        assert_debug_snapshot!(lex(r#"
DO $doblock$
end
$doblock$;"#))
    }

    #[test]
    fn dollar_quote_mismatch_tags_simple() {
        assert_debug_snapshot!(lex(r#"
-- dollar quoting with mismatched tags
$foo$hello world$bar$
"#));
    }

    #[test]
    fn dollar_quote_mismatch_tags_complex() {
        assert_debug_snapshot!(lex(r#"
-- with dollar inside but mismatched tags
$foo$hello$world$bar$
"#));
    }

    #[test]
    fn dollar_quote_closes_only_at_full_tag() {
        // `myfn$` and `$other$` don't close `$fn$`
        assert_debug_snapshot!(lex("$fn$ select 1 /* myfn$ $other$ fn$ $f$ */ $fn$;"), @r#"
        [
            "$fn$ select 1 /* myfn$ $other$ fn$ $f$ */ $fn$" @ Literal { kind: DollarQuotedString { terminated: true } },
            ";" @ Semi,
        ]
        "#);
    }

    #[test]
    fn dollar_quote_unterminated_at_partial_tag() {
        assert_debug_snapshot!(lex("$fn$ select 1 myfn$"), @r#"
        [
            "$fn$ select 1 myfn$" @ Literal { kind: DollarQuotedString { terminated: false } },
        ]
        "#);
    }

    #[test]
    fn numeric() {
        assert_debug_snapshot!(lex(r#"
42
3.5
4.
.001
.123e10
5e2
1.925e-3
1e-10
1e+10
1e10
4664.E+5
"#))
    }

    #[test]
    fn numeric_non_decimal() {
        assert_debug_snapshot!(lex(r#"
0b100101
0B10011001
0o273
0O755
0x42f
0XFFFF
"#))
    }

    /// Found by the `lexer_vs_scan` fuzz target.
    #[test]
    fn numeric_non_decimal_has_no_fraction() {
        assert_debug_snapshot!(lex("SELECT 0b100000.a"), @r#"
        [
            "SELECT" @ Ident,
            " " @ Space,
            "0b100000" @ Literal { kind: Int { base: Binary, empty_int: false } },
            "." @ Dot,
            "a" @ Ident,
        ]
        "#);
    }

    /// Found by the `lexer_vs_scan` fuzz target.
    #[test]
    fn numeric_has_one_fraction() {
        assert_debug_snapshot!(lex("-.5228787.(0"), @r#"
        [
            "-" @ Minus,
            ".5228787" @ Literal { kind: Float { base: Decimal, empty_exponent: false } },
            "." @ Dot,
            "(" @ OpenParen,
            "0" @ Literal { kind: Int { base: Decimal, empty_int: false } },
        ]
        "#);
    }

    #[test]
    fn numeric_with_seperators() {
        assert_debug_snapshot!(lex(r#"
1_500_000_000
0b10001000_00000000
0o_1_755
0xFFFF_FFFF
1.618_034
"#))
    }

    #[test]
    fn select_with_period() {
        assert_debug_snapshot!(lex(r#"
select public.users;
"#))
    }

    #[test]
    fn dot_dot() {
        assert_debug_snapshot!(lex("deep... 1..10 ..5"))
    }

    #[test]
    fn bitstring() {
        assert_debug_snapshot!(lex(r#"
B'1001'
b'1001'
X'1FF'
x'1FF'
"#))
    }

    #[test]
    fn string() {
        assert_debug_snapshot!(lex(r#"
'Dianne''s horse'

select 'foo ''
bar';

select 'foooo'
   'bar';


'foo \\ \n \tbar'

'forgot to close the string
"#))
    }

    #[test]
    fn params() {
        assert_debug_snapshot!(lex(r#"
select $1 + $2;

select $1123123123123;

select $;
"#))
    }

    #[test]
    fn string_with_escapes() {
        // https://www.postgresql.org/docs/current/sql-syntax-lexical.html#SQL-SYNTAX-STRINGS-ESCAPE

        assert_debug_snapshot!(lex(r#"
E'foo'

e'bar'

e'\b\f\n\r\t'

e'\0\11\777'

e'\x0\x11\xFF'

e'\uAAAA \UFFFFFFFF'

"#))
    }

    /// Found by the `lexer_vs_scan` fuzz target.
    #[test]
    fn string_with_escaped_quote() {
        assert_debug_snapshot!(lex(r"trim(E'\'\\000'::bytea)"), @r#"
        [
            "trim" @ Ident,
            "(" @ OpenParen,
            "E'\\'\\\\000'" @ Literal { kind: EscStr { terminated: true } },
            "::" @ DoubleColon,
            "bytea" @ Ident,
            ")" @ CloseParen,
        ]
        "#);
    }

    /// Found by the `lexer_vs_scan` fuzz target.
    #[test]
    fn unicode_prefix_needs_a_quote() {
        assert_debug_snapshot!(lex("U&ER"), @r#"
        [
            "U" @ Ident,
            "&" @ And,
            "ER" @ Ident,
        ]
        "#);
    }

    /// Found by the `lexer_vs_scan` fuzz target.
    #[test]
    fn prefixed_string_after_param_prefix() {
        assert_debug_snapshot!(lex(r":e'Infinity' @x'0080' $U&'\1100' :eb"), @r#"
        [
            ":" @ Colon,
            "e'Infinity'" @ Literal { kind: EscStr { terminated: true } },
            " " @ Space,
            "@" @ At,
            "x'0080'" @ Literal { kind: ByteStr { terminated: true } },
            " " @ Space,
            "$" @ PositionalParam,
            "U&'\\1100'" @ Literal { kind: UnicodeEscStr { terminated: true } },
            " " @ Space,
            ":eb" @ NamedParam { kind: ColonRaw },
        ]
        "#);
    }

    #[test]
    fn string_unicode_escape() {
        // https://www.postgresql.org/docs/current/sql-syntax-lexical.html#SQL-SYNTAX-STRINGS-UESCAPE

        assert_debug_snapshot!(lex(r#"
U&"d\0061t\+000061"

U&"\0441\043B\043E\043D"

u&'\0441\043B'

U&"d!0061t!+000061" UESCAPE '!'
"#))
    }

    #[test]
    fn quoted_ident() {
        assert_debug_snapshot!(lex(r#"
"hello &1 -world";


"hello-world
"#))
    }

    #[test]
    fn quoted_ident_with_escape_quote() {
        assert_debug_snapshot!(lex(r#"
"foo "" bar"
"#))
    }
}
