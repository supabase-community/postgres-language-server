//! The identifier to write in SQL for a name. `pgls_query::Identifier` reads names from identifiers.

use std::borrow::Cow;

use crate::KeywordCategory;

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
    /// reserved ones and those that name columns, which would be the SQL construct of the same
    /// name, like `coalesce(a)` or `interval`, and except `operator`, which starts an
    /// operator like `OPERATOR(pg_catalog.+)` when a `(` follows ([`qual_Op`]).
    ///
    /// [`qual_Op`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L16658
    TypeFunctionName,
    /// A word that may be any keyword except the reserved ones (`NonReservedWord`), such as an
    /// option name of `EXPLAIN (...)`.
    NonReservedWord,
    /// A name after a dot (`attr_name`, a `ColLabel`): any keyword.
    ColLabel,
}

impl NamePosition {
    /// The identifier to write in SQL for `name` here: the name itself if Postgres reads it back
    /// as the same name, else the name quoted, with `"` escaped as `""`. The inverse of
    /// `pgls_query::Identifier::name`.
    pub fn quote(self, name: &str) -> Cow<'_, str> {
        if self.needs_quotes(name) {
            Cow::Owned(format!("\"{}\"", name.replace('"', "\"\"")))
        } else {
            Cow::Borrowed(name)
        }
    }

    /// Whether `name` must be quoted here. Unquoted, it must start with a lowercase ASCII
    /// letter, `_` or a character that isn't ASCII, go on with those or digits, and not be a
    /// keyword that can't be used here.
    ///
    /// Names that aren't ASCII stay unquoted, unlike in Postgres' [`quote_identifier`]: in a
    /// multibyte server encoding such as UTF8, the scanner downcases only ASCII letters
    /// ([`downcase_identifier`]), so `größe` and `Ölstand` read back as written. Like Postgres,
    /// `$` is quoted, though the scanner reads it as part of an identifier.
    ///
    /// [`quote_identifier`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/utils/adt/ruleutils.c#L13079
    /// [`downcase_identifier`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scansup.c#L70
    pub fn needs_quotes(self, name: &str) -> bool {
        let mut chars = name.chars();
        let Some(first) = chars.next() else {
            return true;
        };
        let is_letter = |c: char| c.is_ascii_lowercase() || c == '_' || !c.is_ascii();
        if !is_letter(first) || !chars.all(|c| is_letter(c) || c.is_ascii_digit()) {
            return true;
        }
        if self == NamePosition::TypeFunctionName && name == "operator" {
            return true;
        }
        match KeywordCategory::of(name) {
            None | Some(KeywordCategory::Unreserved) => false,
            Some(KeywordCategory::Reserved) => self != NamePosition::ColLabel,
            Some(KeywordCategory::ColName) => self == NamePosition::TypeFunctionName,
            Some(KeywordCategory::TypeFuncName) => self == NamePosition::ColId,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Keyword;

    #[test]
    fn names_are_quoted_when_postgres_would_read_another_name() {
        let quoted = |name| NamePosition::ColId.quote(name).into_owned();
        assert_eq!(quoted("users"), "users");
        assert_eq!(quoted("_t1"), "_t1");
        assert_eq!(quoted("_"), "_");
        assert_eq!(quoted("t$1"), r#""t$1""#);
        assert_eq!(quoted("Äbc"), "Äbc");
        assert_eq!(quoted("äbc"), "äbc");
        assert_eq!(quoted("MyTable"), r#""MyTable""#);
        assert_eq!(quoted("my table"), r#""my table""#);
        assert_eq!(quoted("1st"), r#""1st""#);
        assert_eq!(quoted(r#"a"b"#), r#""a""b""#);
        assert_eq!(quoted(""), r#""""#);
        for name in [
            "users", "t$1", "Äbc", "äbc", "MyTable", "my table", r#"a"b"#,
        ] {
            assert_eq!(pgls_query::Identifier::new(&quoted(name)).name(), name);
        }
    }

    #[test]
    fn keywords_are_quoted_where_the_grammar_rejects_them() {
        use NamePosition::*;
        // Unreserved: `name`. Column name: `between`. Type or function name: `left`.
        // Reserved: `select`.
        for (name, col_id, type_function_name, non_reserved_word) in [
            ("name", false, false, false),
            ("between", false, true, false),
            ("left", true, false, false),
            ("select", true, true, true),
        ] {
            assert_eq!(ColId.needs_quotes(name), col_id, "{name}");
            assert_eq!(
                TypeFunctionName.needs_quotes(name),
                type_function_name,
                "{name}"
            );
            assert_eq!(
                NonReservedWord.needs_quotes(name),
                non_reserved_word,
                "{name}"
            );
            assert!(!ColLabel.needs_quotes(name), "{name}");
        }
    }

    /// Every keyword, written at each position, parses, e.g. a function named `operator` is
    /// called as `"operator"()`.
    #[test]
    fn keywords_round_trip_through_the_parser() {
        use NamePosition::*;
        let mut failures = Vec::new();
        for keyword in Keyword::ALL.iter().map(|keyword| keyword.name) {
            let col_id = ColId.quote(keyword);
            let function = TypeFunctionName.quote(keyword);
            let word = NonReservedWord.quote(keyword);
            let label = ColLabel.quote(keyword);
            for sql in [
                format!("select {col_id} from {col_id}.t {col_id} where {col_id} = 1"),
                format!("select {col_id}.{label}, s.{label}() from {col_id}"),
                format!("select {function}(), a + {function}(1)"),
                format!("explain ({word}) select 1"),
            ] {
                if pgls_query::parse(&sql).is_err() {
                    failures.push(sql);
                }
            }
        }
        assert!(failures.is_empty(), "{failures:#?}");
        assert_eq!(TypeFunctionName.quote("operator"), r#""operator""#);
        assert_eq!(ColId.quote("operator"), "operator");
    }
}
