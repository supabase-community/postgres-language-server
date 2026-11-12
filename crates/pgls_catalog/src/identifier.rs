//! Identifiers in SQL text and the names they stand for.

use std::borrow::Cow;

use pgls_lexer::{KeywordCategory, keyword_category};

/// `NAMEDATALEN - 1`, the longest name Postgres keeps, in bytes.
///
/// See <https://github.com/postgres/postgres/blob/REL_18_6/src/include/pg_config_manual.h#L29>.
pub const MAX_IDENTIFIER_BYTES: usize = 63;

/// The name an identifier as written in SQL stands for, like Postgres' scanner reads it:
///
/// - a quoted identifier without its quotes and with `""` unescaped,
/// - a `U&"..."` identifier with its Unicode escapes replaced, see [`unicode_identifier_name`].
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

/// The name a `U&"..."` identifier stands for, given the string literal of its `UESCAPE` clause
/// if it has one, or `None` if Postgres rejects the escape character or the escapes. The name is
/// not truncated.
pub fn unicode_identifier_name(text: &str, uescape: Option<&str>) -> Option<String> {
    let escape = match uescape {
        Some(literal) => uescape_char(literal)?,
        None => '\\',
    };
    unicode_identifier_value(text, escape)
}

/// The name a `U&"..."` identifier stands for, with `escape` as its escape character, or `None`
/// if Postgres rejects its escapes. The name is not truncated. Port of [`str_udeescape`].
///
/// [`str_udeescape`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parser.c#L372
fn unicode_identifier_value(text: &str, escape: char) -> Option<String> {
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

/// Where a name is written, which decides the keywords it can be without quotes. See the name
/// classification hierarchy in [`gram.y`].
///
/// [`gram.y`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L17620
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamePosition {
    /// The first part of a column, table or schema name (`ColId`): any keyword except the
    /// reserved ones and those that name types or functions.
    ColId,
    /// An unqualified function or type name (`type_function_name`): any keyword except the
    /// reserved ones and those that name columns, and except `operator`, which starts an
    /// operator like `OPERATOR(pg_catalog.+)` when a `(` follows ([`qual_Op`]).
    ///
    /// [`qual_Op`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L16658
    TypeFunctionName,
    /// A name after a dot (`attr_name`, a `ColLabel`): any keyword.
    ColLabel,
}

/// The identifier to write in SQL for `name`: the name itself if Postgres reads it back as the
/// same name, else the name quoted, with `"` escaped as `""`. The inverse of
/// [`identifier_name`].
pub fn quote_identifier(name: &str, position: NamePosition) -> Cow<'_, str> {
    if needs_quotes(name, position) {
        Cow::Owned(format!("\"{}\"", name.replace('"', "\"\"")))
    } else {
        Cow::Borrowed(name)
    }
}

/// Whether `name` must be quoted at `position`. Unquoted, it must be an identifier by
/// [`scan.l`]'s rules that downcasing leaves unchanged, and not a keyword that can't be used
/// there.
///
/// [`scan.l`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L332
pub fn needs_quotes(name: &str, position: NamePosition) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return true;
    };
    let is_start = |c: char| c.is_ascii_lowercase() || c == '_' || !c.is_ascii();
    if !is_start(first) || !chars.all(|c| is_start(c) || c.is_ascii_digit() || c == '$') {
        return true;
    }
    if position == NamePosition::TypeFunctionName && name == "operator" {
        return true;
    }
    match keyword_category(name) {
        None | Some(KeywordCategory::Unreserved) => false,
        Some(KeywordCategory::Reserved) => position != NamePosition::ColLabel,
        Some(KeywordCategory::ColName) => position == NamePosition::TypeFunctionName,
        Some(KeywordCategory::TypeFuncName) => position == NamePosition::ColId,
    }
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
        assert_eq!(
            unicode_identifier_name(r#"u&"a!!b!0063""#, Some("'!'")).as_deref(),
            Some("a!bc")
        );
        assert_eq!(
            unicode_identifier_name(r#"U&"d\0061ta""#, None).as_deref(),
            Some("data")
        );
        assert_eq!(unicode_identifier_name(r#"U&"a""#, Some("'a'")), None);
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

    #[test]
    fn names_are_quoted_when_postgres_would_read_another_name() {
        let quoted = |name| quote_identifier(name, NamePosition::ColId).into_owned();
        assert_eq!(quoted("users"), "users");
        assert_eq!(quoted("_t1$x"), "_t1$x");
        assert_eq!(quoted("Äbc"), "Äbc");
        assert_eq!(quoted("MyTable"), r#""MyTable""#);
        assert_eq!(quoted("my table"), r#""my table""#);
        assert_eq!(quoted("1st"), r#""1st""#);
        assert_eq!(quoted(r#"a"b"#), r#""a""b""#);
        assert_eq!(quoted(""), r#""""#);
        for name in ["users", "MyTable", "my table", r#"a"b"#] {
            assert_eq!(identifier_name(&quoted(name)), name);
        }
    }

    #[test]
    fn keywords_are_quoted_where_the_grammar_rejects_them() {
        use NamePosition::*;
        // Unreserved: `name`. Column name: `between`. Type or function name: `left`.
        // Reserved: `select`.
        for (name, col_id, type_function_name) in [
            ("name", false, false),
            ("between", false, true),
            ("left", true, false),
            ("select", true, true),
        ] {
            assert_eq!(needs_quotes(name, ColId), col_id, "{name}");
            assert_eq!(
                needs_quotes(name, TypeFunctionName),
                type_function_name,
                "{name}"
            );
            assert!(!needs_quotes(name, ColLabel), "{name}");
        }
    }

    /// Every keyword, written by [`quote_identifier`] at each position, parses, e.g. a function
    /// named `operator` is called as `"operator"()`.
    #[test]
    fn keywords_round_trip_through_the_parser() {
        use NamePosition::*;
        let kwlist = include_str!(
            "../../pgls_query/vendor/libpg_query/src/postgres/include/parser/kwlist.h"
        );
        let keywords = kwlist
            .lines()
            .filter_map(|line| line.strip_prefix("PG_KEYWORD(\""))
            .filter_map(|line| line.split('"').next());
        let mut failures = Vec::new();
        for keyword in keywords {
            let col_id = quote_identifier(keyword, ColId);
            let function = quote_identifier(keyword, TypeFunctionName);
            let label = quote_identifier(keyword, ColLabel);
            for sql in [
                format!("select {col_id} from {col_id}.t {col_id} where {col_id} = 1"),
                format!("select {col_id}.{label}, s.{label}() from {col_id}"),
                format!("select {function}(), a + {function}(1)"),
            ] {
                if pgls_query::parse(&sql).is_err() {
                    failures.push(sql);
                }
            }
        }
        assert!(failures.is_empty(), "{failures:#?}");
        assert_eq!(
            quote_identifier("operator", TypeFunctionName),
            r#""operator""#
        );
        assert_eq!(quote_identifier("operator", ColId), "operator");
    }
}
