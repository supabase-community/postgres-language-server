//! Identifiers in SQL text and the names they stand for.

use std::borrow::Cow;

use pgls_lexer::{KeywordCategory, keyword_category};

/// The name an identifier as written in SQL stands for, like Postgres' scanner reads it: a
/// quoted identifier without its quotes and with `""` unescaped, an unquoted identifier
/// downcased. Like [`downcase_identifier`] in a multibyte encoding such as UTF8, only ASCII
/// letters are downcased. The name is not truncated to 63 bytes.
///
/// Text that is not a complete identifier, like the unterminated `"Foo` of a user who is still
/// typing, is treated as unquoted.
///
/// [`downcase_identifier`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scansup.c#L46
pub fn identifier_name(text: &str) -> String {
    match text
        .strip_prefix('"')
        .and_then(|text| text.strip_suffix('"'))
    {
        Some(quoted) => quoted.replace("\"\"", "\""),
        None => text.to_ascii_lowercase(),
    }
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
    use super::{NamePosition, identifier_name, needs_quotes, quote_identifier};

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
