use std::collections::{HashMap, HashSet};

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamedParameterConversion {
    pub sql: String,
    pub has_identifier_parameters: bool,
    /// Placeholder identifier to the source text it replaced, for instance
    /// `raw_data` to `:raw_data`.
    pub identifier_replacements: HashMap<String, String>,
    /// Positional index to the source text it replaced, for instance `1` to `:'agen_code'`.
    pub value_replacements: HashMap<usize, String>,
    /// False when a placeholder collides with a real identifier of the same statement, in which
    /// case the substitution cannot be undone and the caller must not rewrite the statement.
    pub restorable: bool,
}

/// Converts named parameters in a SQL query string to parser-compatible placeholders.
///
/// Replacements preserve the source byte length so parser diagnostics can be mapped directly
/// back to the original query. A raw colon inside brackets is left untouched because it may be
/// PostgreSQL array-slice syntax rather than a named parameter.
pub fn convert_to_positional_params_with_metadata(text: &str) -> NamedParameterConversion {
    let mut result = String::with_capacity(text.len());
    let mut value_params: HashMap<&str, usize> = HashMap::new();
    let mut value_index = 1;
    let mut bracket_depth = 0_usize;
    let mut has_identifier_parameters = false;

    let lexed = lex(text);

    // Identifiers written as such in the source. A placeholder equal to one of them cannot be
    // told apart from it after formatting, so the conversion is flagged as not restorable.
    let plain_identifiers: HashSet<&str> = lexed
        .tokens()
        .enumerate()
        .filter(|(idx, kind)| *kind == SyntaxKind::IDENT && !lexed.text(*idx).is_empty())
        .map(|(idx, _)| lexed.text(idx))
        .collect();

    let mut identifier_replacements: HashMap<String, String> = HashMap::new();
    let mut value_replacements: HashMap<usize, String> = HashMap::new();
    let mut restorable = true;

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

            if is_identifier {
                has_identifier_parameters = true;
            }

            let replacement = if is_identifier {
                let replacement = identifier_replacement(token_text, token_text.len());

                // Two different parameters landing on the same placeholder, which happens when a
                // name yields no usable identifier and both fall back, cannot be told apart on the
                // way back.
                if plain_identifiers.contains(replacement.as_str())
                    || identifier_replacements
                        .get(&replacement)
                        .is_some_and(|existing| existing != token_text)
                {
                    restorable = false;
                }

                identifier_replacements.insert(replacement.clone(), token_text.to_string());
                replacement
            } else {
                let index = *value_params.entry(token_text).or_insert_with(|| {
                    let index = value_index;
                    value_index += 1;
                    index
                });
                value_replacements.insert(index, token_text.to_string());
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
    NamedParameterConversion {
        sql: result,
        has_identifier_parameters,
        identifier_replacements,
        value_replacements,
        restorable,
    }
}

pub fn convert_to_positional_params(text: &str) -> String {
    convert_to_positional_params_with_metadata(text).sql
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

/// Builds the identifier that stands in for a named parameter used in identifier position.
///
/// Deriving it from the parameter name is what makes the substitution reversible: `:raw_data`
/// becomes `raw_data`, which the formatter can map back. Dropping the leading colon always frees
/// at least one byte, so the replacement fits in the source span, underscore included.
///
/// The name is lowercased because PostgreSQL folds unquoted identifiers, so that is the spelling
/// the printer will emit and the one the reverse map has to be keyed on. A name that happens to be
/// a SQL keyword gets an underscore: `:table` would otherwise yield `table`, which does not parse
/// in identifier position.
fn identifier_replacement(token_text: &str, max_len: usize) -> String {
    let name: String = token_text
        .trim_start_matches([':', '@', '$'])
        .trim_matches('"')
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect::<String>()
        .to_ascii_lowercase();

    if name.is_empty() || name.starts_with(|c: char| c.is_ascii_digit()) {
        return "a".to_string();
    }

    let candidate = if SyntaxKind::from_keyword(&name).is_some() {
        format!("{name}_")
    } else {
        name
    };

    if candidate.len() <= max_len {
        candidate
    } else {
        "a".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
                "select * from schema_.items where id = $1 ",
            ),
            (
                "select * from @schema.items where id = $id",
                "select * from schema_.items where id = $1 ",
            ),
            (
                "select * from $schema.items where id = @id",
                "select * from schema_.items where id = $1 ",
            ),
            (
                "select * from :\"schema\".items",
                "select * from schema_  .items",
            ),
            (
                "cross join :raw_data.migration_infos",
                "cross join raw_data .migration_infos",
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

    #[test]
    fn reports_identifier_parameter_conversion_metadata() {
        let input = "select * from :raw_data.documents where id = :id";
        let conversion = convert_to_positional_params_with_metadata(input);

        assert_eq!(
            conversion.sql,
            "select * from raw_data .documents where id = $1 "
        );
        assert!(conversion.has_identifier_parameters);
        assert_eq!(conversion.sql.len(), input.len());
        assert_eq!(convert_to_positional_params(input), conversion.sql);
    }

    #[test]
    fn value_parameters_and_array_slices_do_not_report_identifier_metadata() {
        let value_input = "select :id, @name, $email, :'status'";
        let value_conversion = convert_to_positional_params_with_metadata(value_input);

        assert!(!value_conversion.has_identifier_parameters);
        assert_eq!(value_conversion.sql.len(), value_input.len());

        let slice_input = "select arr[3:array_upper(arr, 1)]";
        let slice_conversion = convert_to_positional_params_with_metadata(slice_input);

        assert!(!slice_conversion.has_identifier_parameters);
        assert_eq!(slice_conversion.sql, slice_input);
        assert_eq!(slice_conversion.sql.len(), slice_input.len());
    }

    #[test]
    fn identifier_placeholder_is_derived_from_the_name() {
        let conversion = convert_to_positional_params_with_metadata("SELECT x FROM :raw_data.t");

        assert_eq!(conversion.sql, "SELECT x FROM raw_data .t");
        assert_eq!(conversion.sql.len(), "SELECT x FROM :raw_data.t".len());
        assert!(conversion.restorable);
        assert_eq!(
            conversion.identifier_replacements.get("raw_data"),
            Some(&":raw_data".to_string())
        );
    }

    #[test]
    fn value_placeholders_are_mapped_back_to_their_source_text() {
        let conversion =
            convert_to_positional_params_with_metadata("SELECT 1 WHERE a = :'agen_code'");

        assert_eq!(
            conversion.value_replacements.get(&1),
            Some(&":'agen_code'".to_string())
        );
        assert!(conversion.restorable);
    }

    #[test]
    fn a_placeholder_colliding_with_a_real_identifier_is_not_restorable() {
        let conversion =
            convert_to_positional_params_with_metadata("SELECT x FROM raw_data.t, :raw_data.u");

        assert!(!conversion.restorable);
    }

    #[test]
    fn a_name_that_is_a_keyword_gets_an_underscore() {
        let conversion = convert_to_positional_params_with_metadata("SELECT x FROM :table.t");

        // `table` alone does not parse in identifier position, `table_` does, and the underscore
        // fits because the colon was dropped.
        assert_eq!(conversion.sql, "SELECT x FROM table_.t");
        assert_eq!(conversion.sql.len(), "SELECT x FROM :table.t".len());
        assert_eq!(
            conversion.identifier_replacements.get("table_"),
            Some(&":table".to_string())
        );
    }

    #[test]
    fn a_derived_name_is_lowercased() {
        let conversion =
            convert_to_positional_params_with_metadata("GRANT usage ON SCHEMA public TO :DB_ROLE");

        // PostgreSQL folds unquoted identifiers, so `db_role` is what the printer will emit and
        // therefore the only key the reverse map can use.
        assert_eq!(
            conversion.identifier_replacements.get("db_role"),
            Some(&":DB_ROLE".to_string())
        );
    }
}
