//! Identifiers in SQL text and the names Postgres' scanner reads from them.

use crate::protobuf::Token;

/// An identifier as written in SQL text, quoted or not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Identifier<'a> {
    text: &'a str,
    /// The escape character of a `U&"..."` identifier, `None` if Postgres rejects the one its
    /// `UESCAPE` clause names.
    escape: Option<char>,
}

impl<'a> Identifier<'a> {
    /// `NAMEDATALEN - 1`, the longest name Postgres keeps, in bytes. The parser cuts longer
    /// names down, so the syntax tree only has their first 63 bytes.
    ///
    /// See <https://github.com/postgres/postgres/blob/REL_18_6/src/include/pg_config_manual.h#L29>.
    pub const NAME_BYTES_MAX: usize = 63;

    /// The identifier `text`. A `U&"..."` identifier has the escape character `\`, see
    /// [`Self::from_tokens`] for one with a `UESCAPE` clause.
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            escape: Some('\\'),
        }
    }

    /// The identifier of the first of `tokens`, given as the kind and text of each token. A
    /// `U&"..."` identifier has the escape character of the `UESCAPE` clause after it, see
    /// [`Self::unicode_escape`]. Comments are skipped.
    pub fn from_tokens(tokens: impl IntoIterator<Item = (Token, &'a str)>) -> Option<Self> {
        let mut tokens = tokens.into_iter();
        let (kind, text) = tokens.next()?;
        let escape = match kind {
            Token::Uident => Self::unicode_escape(tokens),
            _ => Some('\\'),
        };
        Some(Self { text, escape })
    }

    /// The escape character of the `U&"..."` identifier or `U&'...'` string constant that
    /// `tokens` follow: the character of the `UESCAPE` clause they start with, or `\`. `None` if
    /// Postgres rejects the character. Comments are skipped, like Postgres' parser, which reads
    /// the clause through lookahead in [`base_yylex`].
    ///
    /// [`base_yylex`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parser.c#L111
    pub fn unicode_escape(tokens: impl IntoIterator<Item = (Token, &'a str)>) -> Option<char> {
        let mut tokens = tokens
            .into_iter()
            .filter(|(kind, _)| !matches!(kind, Token::SqlComment | Token::CComment));
        match tokens.next() {
            Some((Token::Uescape, _)) => match tokens.next()? {
                (Token::Sconst, literal) => Self::uescape_char(literal),
                _ => None,
            },
            _ => Some('\\'),
        }
    }

    /// The name the identifier stands for, like Postgres' scanner reads it:
    ///
    /// - a quoted identifier without its quotes and with `""` unescaped,
    /// - a `U&"..."` identifier with its Unicode escapes replaced, see [`Self::unicode_name`].
    /// - an unquoted identifier downcased. Like [`downcase_identifier`] in a multibyte encoding
    ///   such as UTF8, only ASCII letters are downcased.
    ///
    /// The name is truncated to 63 bytes, see [`Self::truncate`]. [`Self::untruncated_name`]
    /// keeps the whole name.
    ///
    /// Text that is not a complete identifier, like the unterminated `"Foo` of a user who is
    /// still typing or a `U&"..."` identifier with an invalid escape, is treated as unquoted.
    ///
    /// [`downcase_identifier`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scansup.c#L46
    pub fn name(self) -> String {
        let mut name = self.untruncated_name();
        let length = Self::truncate(&name).len();
        name.truncate(length);
        name
    }

    /// Like [`Self::name`], but without truncating the name to 63 bytes.
    pub fn untruncated_name(self) -> String {
        let text = self.text;
        if text
            .get(..2)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("u&"))
            && let Some(name) = self.unicode_name()
        {
            return name;
        }
        match text
            .strip_prefix('"')
            .and_then(|text| text.strip_suffix('"'))
        {
            Some(quoted) => quoted.replace("\"\"", "\""),
            None => text.to_ascii_lowercase(),
        }
    }

    /// The name a `U&"..."` identifier stands for, or `None` if Postgres rejects its escape
    /// character or its escapes. The name is not truncated.
    pub fn unicode_name(self) -> Option<String> {
        self.unicode_value(self.escape?)
    }

    /// The longest prefix of whole characters of `name` that fits in 63 bytes, which is the name
    /// Postgres keeps. Port of [`truncate_identifier`], which clips with `pg_mbcliplen` so that
    /// multibyte characters aren't split.
    ///
    /// [`truncate_identifier`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scansup.c#L93
    pub fn truncate(name: &str) -> &str {
        &name[..name.floor_char_boundary(Self::NAME_BYTES_MAX)]
    }

    /// The name a `U&"..."` identifier stands for, with `escape` as its escape character, or
    /// `None` if Postgres rejects its escapes. The name is not truncated. Port of
    /// [`str_udeescape`].
    ///
    /// [`str_udeescape`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parser.c#L372
    fn unicode_value(self, escape: char) -> Option<String> {
        let quoted = self.text.get(2..)?.strip_prefix('"')?.strip_suffix('"')?;
        let value = quoted.replace("\"\"", "\"");
        let mut chars = value.chars().peekable();
        let mut out = String::with_capacity(value.len());
        let mut high_surrogate: Option<u32> = None;
        while let Some(c) = chars.next() {
            if c != escape {
                if high_surrogate.is_some() {
                    return None;
                }
                out.push(c);
                continue;
            }
            if chars.peek() == Some(&escape) {
                if high_surrogate.is_some() {
                    return None;
                }
                chars.next();
                out.push(escape);
                continue;
            }
            let digits = if chars.peek() == Some(&'+') {
                chars.next();
                6
            } else {
                4
            };
            let hex: String = chars.by_ref().take(digits).collect();
            if hex.len() != digits || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
                return None;
            }
            let code = u32::from_str_radix(&hex, 16).ok()?;
            match (high_surrogate.take(), code) {
                (None, 0xD800..=0xDBFF) => high_surrogate = Some(code),
                (Some(high), 0xDC00..=0xDFFF) => out.push(char::from_u32(
                    0x10000 + ((high - 0xD800) << 10) + (code - 0xDC00),
                )?),
                (Some(_), _) | (None, 0xDC00..=0xDFFF) => return None,
                (None, _) => out.push(char::from_u32(code).filter(|c| *c != '\0')?),
            }
        }
        high_surrogate.is_none().then_some(out)
    }

    /// The escape character of a `UESCAPE 'c'` clause, given the text of its string literal.
    /// Port of [`check_uescapechar`].
    ///
    /// [`check_uescapechar`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parser.c#L352
    fn uescape_char(literal: &str) -> Option<char> {
        let value = literal
            .strip_prefix('\'')?
            .strip_suffix('\'')?
            .replace("''", "'");
        let mut chars = value.chars();
        let escape = chars.next()?;
        (chars.next().is_none()
            && escape.is_ascii()
            && !escape.is_ascii_hexdigit()
            && !matches!(
                escape,
                '+' | '\'' | '"' | ' ' | '\t' | '\n' | '\r' | '\x0b' | '\x0c'
            ))
        .then_some(escape)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unquoted_identifiers_are_downcased() {
        let name = |text: &str| Identifier::new(text).name();
        assert_eq!(name("MyTable"), "mytable");
        assert_eq!(name("my_table$1"), "my_table$1");
        // Only ASCII letters, like Postgres in a UTF8 database.
        assert_eq!(name("ÄBC"), "Äbc");
    }

    #[test]
    fn quoted_identifiers_keep_their_case() {
        let name = |text: &str| Identifier::new(text).name();
        assert_eq!(name(r#""MyTable""#), "MyTable");
        assert_eq!(name(r#""a""b""#), r#"a"b"#);
        assert_eq!(name(r#""""#), "");
    }

    #[test]
    fn incomplete_quoted_identifiers_are_unquoted() {
        let name = |text: &str| Identifier::new(text).name();
        assert_eq!(name(r#""Foo"#), r#""foo"#);
        assert_eq!(name(r#"""#), r#"""#);
        assert_eq!(name(r#"U&"\00""#), r#"u&"\00""#);
    }

    #[test]
    fn unicode_identifiers_are_unescaped() {
        let name = |text: &str| Identifier::new(text).name();
        let unicode = |text, escape| Identifier::new(text).unicode_value(escape);
        assert_eq!(name(r#"U&"d\0061t\+000061""#), "data");
        assert_eq!(name(r#"u&"Caf\00E9""#), "Café");
        assert_eq!(name(r#"U&"a""b\\""#), r#"a"b\"#);
        assert_eq!(unicode(r#"u&"a!!b!0063""#, '!').as_deref(), Some("a!bc"));
        assert_eq!(unicode(r#"U&"\D83D\DE00""#, '\\').as_deref(), Some("😀"));
        assert_eq!(unicode(r#"U&"\D83D""#, '\\'), None);
        assert_eq!(unicode(r#"U&"\00""#, '\\'), None);
        assert_eq!(unicode(r#"U&"\0000""#, '\\'), None);
        assert_eq!(Identifier::uescape_char("'!'"), Some('!'));
        assert_eq!(Identifier::uescape_char("'a'"), None);
        assert_eq!(Identifier::uescape_char("'!!'"), None);
        assert_eq!(
            Identifier::new(r#"U&"d\0061ta""#).unicode_name().as_deref(),
            Some("data")
        );
    }

    #[test]
    fn unicode_identifiers_take_the_escape_of_their_uescape_clause() {
        let name = |sql: &str| {
            let scan = crate::scan(sql).ok()?;
            let identifier = scan
                .tokens
                .iter()
                .position(|token| token.token() == Token::Uident)?;
            let identifier = Identifier::from_tokens(
                scan.tokens[identifier..]
                    .iter()
                    .map_while(|token| Some((token.token(), token.text(sql)?))),
            )?;
            Some((identifier.name(), identifier.unicode_name()))
        };
        let data = || Some(("data".to_owned(), Some("data".to_owned())));
        assert_eq!(name(r#"select U&"d!0061ta" UESCAPE '!'"#), data());
        assert_eq!(name(r#"select U&"d!0061ta" /* c */ uescape '!'"#), data());
        assert_eq!(name("select U&\"d!0061ta\" -- c\n UESCAPE '!'"), data());
        assert_eq!(name(r#"select U&"d\0061ta" as x"#), data());
        // Postgres rejects the escape character, so the text is read as unquoted.
        assert_eq!(
            name(r#"select U&"a" UESCAPE 'a'"#),
            Some((r#"u&"a""#.to_owned(), None))
        );
    }

    #[test]
    fn names_are_truncated_to_63_bytes() {
        let name = |text: &str| Identifier::new(text).name();
        let long = "a".repeat(70);
        assert_eq!(name(&long), "a".repeat(63));
        assert_eq!(name(&format!("\"{long}\"")), "a".repeat(63));
        assert_eq!(Identifier::new(&long).untruncated_name(), long);
        // A character that doesn't fit is dropped as a whole.
        let multibyte = format!("{}é", "a".repeat(62));
        assert_eq!(name(&multibyte), "a".repeat(62));
        // Unicode escapes are counted after unescaping.
        let escaped = format!("U&\"{}\\00E9\"", "a".repeat(61));
        assert_eq!(name(&escaped), format!("{}é", "a".repeat(61)));
    }
}
