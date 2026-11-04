//! Identifiers in SQL text and the names they stand for.

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

#[cfg(test)]
mod tests {
    use super::identifier_name;

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
}
