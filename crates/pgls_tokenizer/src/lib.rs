mod class;
mod cursor;
mod identifier;
mod keyword;
mod number;
mod operator;
mod psql;
mod quoted;
mod token;
mod trivia;

pub use class::CharClass;
use cursor::Cursor;
pub use identifier::NamePosition;
pub use keyword::{Keyword, KeywordCategory};
use quoted::Quote;
pub use token::{Base, LiteralKind, NamedParamKind, Token, TokenKind};

impl Cursor<'_> {
    // see: https://github.com/rust-lang/rust/blob/ba1d7f4a083e6402679105115ded645512a7aea8/compiler/rustc_lexer/src/lib.rs#L339
    pub(crate) fn advance_token(&mut self) -> Token {
        let rest = self.as_str();
        let Some(first_char) = self.bump() else {
            return Token::new(TokenKind::Eof, 0);
        };
        let token_kind = match first_char {
            // Comments, before the operators that start with `/` or `-`.
            '/' if self.first() == '*' => self.block_comment(),
            '-' if self.first() == '-' => self.line_comment(),

            // Whitespace, a token per kind of character:
            // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L208-L210
            ' ' => {
                self.eat_while(|c| c == ' ');
                TokenKind::Space
            }
            '\t' => {
                self.eat_while(|c| c == '\t');
                TokenKind::Tab
            }
            '\u{000B}' => {
                self.eat_while(|c| c == '\u{000B}');
                TokenKind::VerticalTab
            }
            '\u{000C}' => {
                self.eat_while(|c| c == '\u{000C}');
                TokenKind::FormFeed
            }
            c if CharClass::Newline.contains(c) => self.line_ending_sequence(c),

            '\'' => self.quoted(Quote::Xq),
            '"' => self.quoted(Quote::Xd),
            c if CharClass::IdentStart.contains(c) => match Quote::prefixed(rest) {
                Some(quote) => self.quoted(quote),
                None => self.ident(c),
            },

            c @ '0'..='9' => TokenKind::Literal {
                kind: self.number(c),
            },
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
            ':' => match self.first() {
                ':' => {
                    self.bump();
                    TokenKind::DoubleColon
                }
                // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L340
                // colon_equals	":="
                '=' => {
                    self.bump();
                    TokenKind::ColonEquals
                }
                _ => self.psql_variable(),
            },
            '@' if self.at_identifier() => self.at_param(),
            c if CharClass::OpChar.contains(c) => self.operator(c),
            '$' => self.dollar(),
            '\\' => self.backslash(),

            // One-symbol tokens.
            ';' => TokenKind::Semi,
            ',' => TokenKind::Comma,
            '(' => TokenKind::OpenParen,
            ')' => TokenKind::CloseParen,
            '[' => TokenKind::OpenBracket,
            ']' => TokenKind::CloseBracket,
            _ => TokenKind::Unknown,
        };
        let res = Token::new(token_kind, self.pos_within_token());
        self.reset_pos_within_token();
        res
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
    fn named_param_colon_defined() {
        let result = lex("select :{?flag}");
        assert_debug_snapshot!(result, @r#"
        [
            "select" @ Ident,
            " " @ Space,
            ":{?flag}" @ NamedParam { kind: ColonDefined },
        ]
        "#);
    }

    #[test]
    fn psql_variable_names_are_variable_characters() {
        // psql reads a variable only when its name is variable characters, so the `:` of the
        // others is a colon, and `$` ends the name.
        let result = lex(":'a b' :\"\" :{?} :{x} :a$b :1");
        assert_debug_snapshot!(result, @r#"
        [
            ":" @ Colon,
            "'a b'" @ Literal { kind: Str { terminated: true } },
            " " @ Space,
            ":" @ Colon,
            "\"\"" @ QuotedIdent { terminated: true },
            " " @ Space,
            ":" @ Colon,
            "{" @ Unknown,
            "?" @ Question,
            "}" @ Unknown,
            " " @ Space,
            ":" @ Colon,
            "{" @ Unknown,
            "x" @ Ident,
            "}" @ Unknown,
            " " @ Space,
            ":a" @ NamedParam { kind: ColonRaw },
            "$b" @ NamedParam { kind: DollarRaw },
            " " @ Space,
            ":1" @ NamedParam { kind: ColonRaw },
        ]
        "#);
    }

    #[test]
    fn meta_commands() {
        let result = lex(concat!(
            "\\set x 1\n",
            "select :x \\gset\n",
            "\\echo don't\n",
            "\\echo 'a \\\\ b' \"c \\ d\" `e \\ f` \\\\ select 1;\n",
            "\\if :c \\echo a\n",
            "\\copy t from stdin \\\\ x\n",
            "\\g | cat \\\\ x\n",
            "\\endif",
        ));
        assert_debug_snapshot!(result);
    }

    #[test]
    fn backslash_semicolon_and_colon_are_sql() {
        let result = lex("select 1 \\; select 2\\: x");
        assert_debug_snapshot!(result, @r#"
        [
            "select" @ Ident,
            " " @ Space,
            "1" @ Literal { kind: Int { base: Decimal, empty_int: false } },
            " " @ Space,
            "\\" @ Backslash,
            ";" @ Semi,
            " " @ Space,
            "select" @ Ident,
            " " @ Space,
            "2" @ Literal { kind: Int { base: Decimal, empty_int: false } },
            "\\" @ Backslash,
            ":" @ Colon,
            " " @ Space,
            "x" @ Ident,
        ]
        "#);
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

    /// A run of operator characters is one operator. It stops at a comment and before `@name`,
    /// and puts back a trailing `+` or `-` unless it has a character no SQL operator has.
    #[test]
    fn operator_runs() {
        assert_debug_snapshot!(lex("a<=b ~* 1=-1 ?- a+/*c*/b <->c=@p @@x a<--c"), @r#"
        [
            "a" @ Ident,
            "<=" @ Operator,
            "b" @ Ident,
            " " @ Space,
            "~*" @ Operator,
            " " @ Space,
            "1" @ Literal { kind: Int { base: Decimal, empty_int: false } },
            "=" @ Eq,
            "-" @ Minus,
            "1" @ Literal { kind: Int { base: Decimal, empty_int: false } },
            " " @ Space,
            "?-" @ Operator,
            " " @ Space,
            "a" @ Ident,
            "+" @ Plus,
            "/*c*/" @ BlockComment { terminated: true },
            "b" @ Ident,
            " " @ Space,
            "<->" @ Operator,
            "c" @ Ident,
            "=" @ Eq,
            "@p" @ NamedParam { kind: AtPrefix },
            " " @ Space,
            "@" @ At,
            "@x" @ NamedParam { kind: AtPrefix },
            " " @ Space,
            "a" @ Ident,
            "<" @ Lt,
            "--c" @ LineComment,
        ]
        "#);
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

    /// Found by the `lexer_vs_scan` fuzz target.
    #[test]
    fn colon_string_continues_on_the_next_line() {
        assert_debug_snapshot!(lex(":'line'\n' - next'\n\t' - third' 'x'"), @r#"
        [
            ":'line'\n' - next'\n\t' - third'" @ NamedParam { kind: ColonString },
            " " @ Space,
            "'x'" @ Literal { kind: Str { terminated: true } },
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
