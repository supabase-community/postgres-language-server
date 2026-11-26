use std::collections::HashMap;

use pgls_query::{Identifier, protobuf::Token};

use crate::source::Source;

/// The identifiers of the statement that are longer than Postgres keeps, by the name the parser cut them
/// down to.
///
/// The formatter writes names from the AST. Without this, it would rewrite a long identifier to its
/// truncated form, which means the same to Postgres but is not what was written. When two
/// different spellings are cut to the same name, neither is restored.
pub(crate) fn truncated_identifiers(source: &Source<'_>) -> HashMap<String, String> {
    let mut by_truncated: HashMap<String, Option<String>> = HashMap::new();
    for token in &source.tokens {
        if Token::try_from(token.token) != Ok(Token::Ident) {
            continue;
        }
        let Some(text) = source.sql.get(token.start as usize..token.end as usize) else {
            continue;
        };
        let name = Identifier::new(text).untruncated_name();
        // A name of exactly the maximum length takes part too: written next to a longer one with
        // the same prefix, it is not clear which spelling to restore.
        if name.len() < Identifier::NAME_BYTES_MAX {
            continue;
        }

        let truncated = Identifier::truncate(&name).to_string();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_long_identifier_is_found_by_its_truncated_name() {
        let long = "a_role_name_that_is_definitely_way_too_long_for_anything_postgres_keeps";
        let identifiers = truncated_identifiers(&Source::scan(&format!("grant r to {long}")));

        assert_eq!(identifiers.get(&long[..63]).map(String::as_str), Some(long));
        assert_eq!(identifiers.len(), 1);
    }

    #[test]
    fn a_quoted_identifier_keeps_its_case_and_spaces() {
        let long = "A Quoted Name That Is Definitely Way Too Long For Anything Postgres Keeps";
        let identifiers = truncated_identifiers(&Source::scan(&format!("select 1 as \"{long}\"")));

        assert_eq!(identifiers.get(&long[..63]).map(String::as_str), Some(long));
    }

    #[test]
    fn a_multibyte_character_is_not_split() {
        // 62 ASCII bytes followed by a two-byte character: Postgres keeps the 62 bytes.
        let long = format!("{}é_and_more", "a".repeat(62));
        let identifiers = truncated_identifiers(&Source::scan(&format!("select 1 as {long}")));

        assert_eq!(
            identifiers.get(&"a".repeat(62)).map(String::as_str),
            Some(long.as_str())
        );
    }

    #[test]
    fn two_identifiers_cut_to_the_same_name_are_not_restored() {
        let prefix = "x".repeat(63);
        let identifiers =
            truncated_identifiers(&Source::scan(&format!("select {prefix}_one, {prefix}_two")));

        assert!(identifiers.is_empty());
    }

    #[test]
    fn a_name_written_both_truncated_and_in_full_is_not_restored() {
        let prefix = "x".repeat(63);
        let identifiers =
            truncated_identifiers(&Source::scan(&format!("select {prefix}, {prefix}_long")));

        assert!(identifiers.is_empty());
    }

    #[test]
    fn short_identifiers_are_ignored() {
        assert!(truncated_identifiers(&Source::scan("select a, \"B\" from t")).is_empty());
    }
}
