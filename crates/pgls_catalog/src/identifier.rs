//! Identifiers in SQL text and the names they stand for.

/// `NAMEDATALEN - 1`, the longest name Postgres keeps, in bytes.
///
/// See <https://github.com/postgres/postgres/blob/REL_18_6/src/include/pg_config_manual.h#L29>.
pub const MAX_IDENTIFIER_BYTES: usize = 63;

/// The name an identifier as written in SQL stands for, like Postgres' scanner reads it:
///
/// - a quoted identifier without its quotes and with `""` unescaped,
/// - a `U&"..."` identifier with its Unicode escapes replaced, see [`unicode_identifier_value`].
///   A `UESCAPE` clause after the identifier isn't part of `text`, so the escape character is
///   `\`.
/// - an unquoted identifier downcased. Like [`downcase_identifier`] in a multibyte encoding such
///   as UTF8, only ASCII letters are downcased.
///
/// The name is truncated to 63 bytes, see [`truncate_identifier`]. [`untruncated_identifier_name`]
/// keeps the whole name.
///
/// Text that is not a complete identifier, like the unterminated `"Foo` of a user who is still
/// typing or a `U&"..."` identifier with an invalid escape, is treated as unquoted.
///
/// [`downcase_identifier`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scansup.c#L46
pub fn identifier_name(text: &str) -> String {
    let mut name = untruncated_identifier_name(text);
    let length = truncate_identifier(&name).len();
    name.truncate(length);
    name
}

/// Like [`identifier_name`], but without truncating the name to 63 bytes.
pub fn untruncated_identifier_name(text: &str) -> String {
    if text
        .get(..2)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("u&"))
        && let Some(name) = unicode_identifier_value(text, '\\')
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

/// The longest prefix of whole characters of `name` that fits in 63 bytes, which is the name
/// Postgres keeps. Port of [`truncate_identifier`], which clips with `pg_mbcliplen` so that
/// multibyte characters aren't split.
///
/// [`truncate_identifier`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scansup.c#L93
pub fn truncate_identifier(name: &str) -> &str {
    &name[..name.floor_char_boundary(MAX_IDENTIFIER_BYTES)]
}

/// The name a `U&"..."` identifier stands for, with `escape` as its escape character, or `None`
/// if Postgres rejects its escapes. The name is not truncated. Port of [`str_udeescape`].
///
/// [`str_udeescape`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parser.c#L372
pub fn unicode_identifier_value(text: &str, escape: char) -> Option<String> {
    let quoted = text.get(2..)?.strip_prefix('"')?.strip_suffix('"')?;
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

/// The escape character of a `UESCAPE 'c'` clause, given the text of its string literal. Port of
/// [`check_uescapechar`].
///
/// [`check_uescapechar`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parser.c#L352
pub fn uescape_char(literal: &str) -> Option<char> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unquoted_identifiers_are_downcased() {
        assert_eq!(identifier_name("MyTable"), "mytable");
        assert_eq!(identifier_name("my_table$1"), "my_table$1");
        // Only ASCII letters, like Postgres in a UTF8 database.
        assert_eq!(identifier_name("ÄBC"), "Äbc");
    }

    #[test]
    fn quoted_identifiers_keep_their_case() {
        assert_eq!(identifier_name(r#""MyTable""#), "MyTable");
        assert_eq!(identifier_name(r#""a""b""#), r#"a"b"#);
        assert_eq!(identifier_name(r#""""#), "");
    }

    #[test]
    fn incomplete_quoted_identifiers_are_unquoted() {
        assert_eq!(identifier_name(r#""Foo"#), r#""foo"#);
        assert_eq!(identifier_name(r#"""#), r#"""#);
        assert_eq!(identifier_name(r#"U&"\00""#), r#"u&"\00""#);
    }

    #[test]
    fn unicode_identifiers_are_unescaped() {
        assert_eq!(identifier_name(r#"U&"d\0061t\+000061""#), "data");
        assert_eq!(identifier_name(r#"u&"Caf\00E9""#), "Café");
        assert_eq!(identifier_name(r#"U&"a""b\\""#), r#"a"b\"#);
        assert_eq!(
            unicode_identifier_value(r#"u&"a!!b!0063""#, '!').as_deref(),
            Some("a!bc")
        );
        assert_eq!(
            unicode_identifier_value(r#"U&"\D83D\DE00""#, '\\').as_deref(),
            Some("😀")
        );
        assert_eq!(unicode_identifier_value(r#"U&"\D83D""#, '\\'), None);
        assert_eq!(unicode_identifier_value(r#"U&"\00""#, '\\'), None);
        assert_eq!(unicode_identifier_value(r#"U&"\0000""#, '\\'), None);
        assert_eq!(uescape_char("'!'"), Some('!'));
        assert_eq!(uescape_char("'a'"), None);
        assert_eq!(uescape_char("'!!'"), None);
    }

    #[test]
    fn names_are_truncated_to_63_bytes() {
        let long = "a".repeat(70);
        assert_eq!(identifier_name(&long), "a".repeat(63));
        assert_eq!(identifier_name(&format!("\"{long}\"")), "a".repeat(63));
        assert_eq!(untruncated_identifier_name(&long), long);
        // A character that doesn't fit is dropped as a whole.
        let multibyte = format!("{}é", "a".repeat(62));
        assert_eq!(identifier_name(&multibyte), "a".repeat(62));
        // Unicode escapes are counted after unescaping.
        let escaped = format!("U&\"{}\\00E9\"", "a".repeat(61));
        assert_eq!(identifier_name(&escaped), format!("{}é", "a".repeat(61)));
    }
}
