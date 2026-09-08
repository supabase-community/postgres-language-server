use std::collections::HashMap;

use crate::{SyntaxKind, lex};

// Keywords that, when preceding a named parameter, indicate that the parameter should be treated
// as an identifier rather than a positional parameter.
const IDENTIFIER_CONTEXT: [SyntaxKind; 15] = [
    SyntaxKind::TO_KW,
    SyntaxKind::FROM_KW,
    SyntaxKind::SCHEMA_KW,
    SyntaxKind::TABLE_KW,
    SyntaxKind::INDEX_KW,
    SyntaxKind::CONSTRAINT_KW,
    SyntaxKind::OWNER_KW,
    SyntaxKind::ROLE_KW,
    SyntaxKind::USER_KW,
    SyntaxKind::DATABASE_KW,
    SyntaxKind::TYPE_KW,
    SyntaxKind::CAST_KW,
    SyntaxKind::ALTER_KW,
    SyntaxKind::DROP_KW,
    // for schema.table style identifiers
    SyntaxKind::DOT,
];

/// Converts named parameters in a SQL query string to parser-compatible placeholders.
///
/// Replacements preserve the source byte length so parser diagnostics can be mapped directly
/// back to the original query. A raw colon inside brackets is left untouched because it may be
/// PostgreSQL array-slice syntax rather than a named parameter.
pub fn convert_to_positional_params(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut value_params: HashMap<&str, usize> = HashMap::new();
    let mut identifier_params: HashMap<&str, usize> = HashMap::new();
    let mut value_index = 1;
    let mut identifier_index = 0;
    let mut bracket_depth = 0_usize;

    let lexed = lex(text);
    for (token_idx, kind) in lexed.tokens().enumerate() {
        if kind == SyntaxKind::EOF {
            break;
        }

        let token_text = lexed.text(token_idx);

        if matches!(kind, SyntaxKind::NAMED_PARAM) {
            let flavor = NamedParamFlavor::from_text(token_text);
            let previous = previous_non_trivia_kind(&lexed, token_idx);
            let next = next_non_trivia_kind(&lexed, token_idx);
            let is_identifier = match flavor {
                Some(NamedParamFlavor::ColonRaw) if bracket_depth > 0 => {
                    result.push_str(token_text);
                    continue;
                }
                Some(NamedParamFlavor::ColonIdentifier) => true,
                _ if next == Some(SyntaxKind::DOT) => true,
                _ if previous.is_some_and(|kind| IDENTIFIER_CONTEXT.contains(&kind)) => true,
                _ => false,
            };

            let replacement = if is_identifier {
                let index = *identifier_params.entry(token_text).or_insert_with(|| {
                    let index = identifier_index;
                    identifier_index += 1;
                    index
                });
                identifier_replacement(index, token_text.len())
            } else {
                let index = *value_params.entry(token_text).or_insert_with(|| {
                    let index = value_index;
                    value_index += 1;
                    index
                });
                format!("${index}")
            };

            push_padded_replacement(&mut result, &replacement, token_text.len());
        } else {
            result.push_str(token_text);
        }

        match kind {
            SyntaxKind::L_BRACK => bracket_depth += 1,
            SyntaxKind::R_BRACK => bracket_depth = bracket_depth.saturating_sub(1),
            _ => {}
        }
    }

    debug_assert_eq!(result.len(), text.len());
    result
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NamedParamFlavor {
    AtPrefix,
    DollarRaw,
    ColonRaw,
    ColonString,
    ColonIdentifier,
}

impl NamedParamFlavor {
    fn from_text(text: &str) -> Option<Self> {
        match text.as_bytes().first()? {
            b'@' => Some(Self::AtPrefix),
            b'$' => Some(Self::DollarRaw),
            b':' if text.starts_with(":'") => Some(Self::ColonString),
            b':' if text.starts_with(":\"") => Some(Self::ColonIdentifier),
            b':' => Some(Self::ColonRaw),
            _ => None,
        }
    }
}

fn previous_non_trivia_kind(lexed: &crate::Lexed<'_>, token_idx: usize) -> Option<SyntaxKind> {
    (0..token_idx)
        .rev()
        .map(|idx| lexed.kind(idx))
        .find(|kind| !kind.is_trivia())
}

fn next_non_trivia_kind(lexed: &crate::Lexed<'_>, token_idx: usize) -> Option<SyntaxKind> {
    (token_idx + 1..lexed.len())
        .map(|idx| lexed.kind(idx))
        .find(|kind| !kind.is_trivia() && *kind != SyntaxKind::EOF)
}

fn push_padded_replacement(result: &mut String, replacement: &str, original_len: usize) {
    assert!(
        replacement.len() <= original_len,
        "named parameter replacement must preserve source length"
    );
    result.push_str(replacement);
    result.push_str(&" ".repeat(original_len - replacement.len()));
}

fn identifier_replacement(index: usize, max_len: usize) -> String {
    let identifier = deterministic_identifier(index);
    if identifier.len() <= max_len {
        identifier
    } else {
        "a".to_string()
    }
}

const ALPHABET: [char; 26] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z',
];

/// Generates a deterministic identifier based on the given index.
fn deterministic_identifier(idx: usize) -> String {
    let iteration = idx / ALPHABET.len();
    let pos = idx % ALPHABET.len();

    format!(
        "{}{}",
        ALPHABET[pos],
        if iteration > 0 {
            deterministic_identifier(iteration - 1)
        } else {
            "".to_string()
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_identifier() {
        assert_eq!(deterministic_identifier(0), "a");
        assert_eq!(deterministic_identifier(25), "z");
        assert_eq!(deterministic_identifier(26), "aa");
        assert_eq!(deterministic_identifier(27), "ba");
        assert_eq!(deterministic_identifier(51), "za");
    }

    #[test]
    fn test_convert_to_positional_params() {
        let input = "select * from users where id = @one and name = :two and email = :'three';";
        let result = convert_to_positional_params(input);
        assert_eq!(
            result,
            "select * from users where id = $1   and name = $2   and email = $3      ;"
        );
    }

    #[test]
    fn test_convert_to_positional_params_with_duplicates() {
        let input = "select * from users where first_name = @one and starts_with(email, @one) and created_at > @two;";
        let result = convert_to_positional_params(input);
        assert_eq!(
            result,
            "select * from users where first_name = $1   and starts_with(email, $1  ) and created_at > $2  ;"
        );
    }

    #[test]
    fn classifies_named_parameter_contexts() {
        let cases = [
            (
                "select * from :schema.items where id = :id",
                "select * from a      .items where id = $1 ",
            ),
            (
                "select * from @schema.items where id = $id",
                "select * from a      .items where id = $1 ",
            ),
            (
                "select * from $schema.items where id = @id",
                "select * from a      .items where id = $1 ",
            ),
            (
                "select * from :\"schema\".items",
                "select * from a        .items",
            ),
            (
                "cross join :raw_data.migration_infos",
                "cross join a        .migration_infos",
            ),
        ];

        for (input, expected) in cases {
            let normalized = convert_to_positional_params(input);
            assert_eq!(normalized, expected);
            assert_eq!(normalized.len(), input.len());
        }
    }

    #[test]
    fn preserves_array_slices_and_keeps_parameter_indexes_contiguous() {
        let input = "select arr[3:array_upper(arr, 1)], :value from :schema.items where id = :id";
        let normalized = convert_to_positional_params(input);

        assert!(normalized.contains("arr[3:array_upper(arr, 1)]"));
        assert!(normalized.contains("$1    "));
        assert!(normalized.contains("$2 "));
        assert_eq!(normalized.len(), input.len());
    }

    #[test]
    fn preserves_quoted_and_at_parameters_in_brackets() {
        let input = "select arr[:'value'], arr[@value], arr[$value]";
        let normalized = convert_to_positional_params(input);

        assert_eq!(normalized, "select arr[$1      ], arr[$2    ], arr[$3    ]");
        assert_eq!(normalized.len(), input.len());
    }
}
