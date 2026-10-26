use std::collections::HashMap;

use pgls_query::protobuf::Token;

/// The longest identifier Postgres keeps, `NAMEDATALEN - 1` bytes. The parser cuts longer ones
/// down, so the AST only has their first 63 bytes.
const MAX_IDENTIFIER_LENGTH: usize = 63;

/// The identifiers of `sql` that are longer than Postgres keeps, by the name the parser cut them
/// down to.
///
/// The formatter writes names from the AST. Without this, it would rewrite a long identifier to its
/// truncated form, which means the same to Postgres but is not what was written. When two
/// different spellings are cut to the same name, neither is restored.
pub(crate) fn truncated_identifiers(sql: &str) -> HashMap<String, String> {
    let Ok(scan) = pgls_query::scan(sql) else {
        return HashMap::new();
    };

    let mut by_truncated: HashMap<String, Option<String>> = HashMap::new();
    for token in &scan.tokens {
        if Token::try_from(token.token) != Ok(Token::Ident) {
            continue;
        }
        let Some(text) = sql.get(token.start as usize..token.end as usize) else {
            continue;
        };
        let name = identifier_name(text);
        // A name of exactly the maximum length takes part too: written next to a longer one with
        // the same prefix, it is not clear which spelling to restore.
        if name.len() < MAX_IDENTIFIER_LENGTH {
            continue;
        }

        let truncated = truncate(&name).to_string();
        by_truncated
            .entry(truncated)
            .and_modify(|existing| {
                if existing.as_deref() != Some(name.as_str()) {
                    *existing = None;
                }
            })
            .or_insert(Some(name));
    }

    by_truncated
        .into_iter()
        .filter_map(|(truncated, name)| Some((truncated, name?)))
        .filter(|(truncated, name)| truncated != name)
        .collect()
}

/// The name an identifier token stands for: a quoted identifier without its quotes, an unquoted
/// one in lower case, as `downcase_identifier` does for ASCII letters.
fn identifier_name(text: &str) -> String {
    match text
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
    {
        Some(quoted) => quoted.replace("\"\"", "\""),
        None => text.to_ascii_lowercase(),
    }
}

/// The longest prefix of whole characters that fits in `MAX_IDENTIFIER_LENGTH` bytes, as
/// `truncate_identifier` keeps it.
fn truncate(name: &str) -> &str {
    let mut end = MAX_IDENTIFIER_LENGTH;
    while !name.is_char_boundary(end) {
        end -= 1;
    }
    &name[..end]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_long_identifier_is_found_by_its_truncated_name() {
        let long = "a_role_name_that_is_definitely_way_too_long_for_anything_postgres_keeps";
        let identifiers = truncated_identifiers(&format!("grant r to {long}"));

        assert_eq!(identifiers.get(&long[..63]).map(String::as_str), Some(long));
        assert_eq!(identifiers.len(), 1);
    }

    #[test]
    fn a_quoted_identifier_keeps_its_case_and_spaces() {
        let long = "A Quoted Name That Is Definitely Way Too Long For Anything Postgres Keeps";
        let identifiers = truncated_identifiers(&format!("select 1 as \"{long}\""));

        assert_eq!(identifiers.get(&long[..63]).map(String::as_str), Some(long));
    }

    #[test]
    fn a_multibyte_character_is_not_split() {
        // 62 ASCII bytes followed by a two-byte character: Postgres keeps the 62 bytes.
        let long = format!("{}é_and_more", "a".repeat(62));
        let identifiers = truncated_identifiers(&format!("select 1 as {long}"));

        assert_eq!(
            identifiers.get(&"a".repeat(62)).map(String::as_str),
            Some(long.as_str())
        );
    }

    #[test]
    fn two_identifiers_cut_to_the_same_name_are_not_restored() {
        let prefix = "x".repeat(63);
        let identifiers = truncated_identifiers(&format!("select {prefix}_one, {prefix}_two"));

        assert!(identifiers.is_empty());
    }

    #[test]
    fn a_name_written_both_truncated_and_in_full_is_not_restored() {
        let prefix = "x".repeat(63);
        let identifiers = truncated_identifiers(&format!("select {prefix}, {prefix}_long"));

        assert!(identifiers.is_empty());
    }

    #[test]
    fn short_identifiers_are_ignored() {
        assert!(truncated_identifiers("select a, \"B\" from t").is_empty());
    }
}
