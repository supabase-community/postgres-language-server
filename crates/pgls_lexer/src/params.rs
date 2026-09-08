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

/// Converts named parameters in a SQL query string to positional parameters.
///
/// This function scans the input SQL string for named parameters (e.g., `@param`, `:param`, `:'param'`)
/// and replaces them with positional parameters (e.g., `$1`, `$2`, etc.).
///
/// It maintains the original spacing of the named parameters in the output string.
///
/// Useful for preparing SQL queries for parsing or execution where named paramters are not supported.
pub fn convert_to_positional_params(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut param_mapping: HashMap<&str, usize> = HashMap::new();
    let mut param_index = 1;

    let lexed = lex(text);
    for (token_idx, kind) in lexed.tokens().enumerate() {
        if kind == SyntaxKind::EOF {
            break;
        }

        let token_text = lexed.text(token_idx);

        if matches!(kind, SyntaxKind::NAMED_PARAM) {
            let idx = match param_mapping.get(token_text) {
                Some(&index) => index,
                None => {
                    let index = param_index;
                    param_mapping.insert(token_text, index);
                    param_index += 1;
                    index
                }
            };

            // find previous non-trivia token
            let prev_token = (0..token_idx)
                .rev()
                .map(|i| lexed.kind(i))
                .find(|kind| !kind.is_trivia());

            let replacement = match prev_token {
                Some(k) if IDENTIFIER_CONTEXT.contains(&k) => deterministic_identifier(idx - 1),
                _ => format!("${idx}"),
            };
            let original_len = token_text.len();
            let replacement_len = replacement.len();

            result.push_str(&replacement);

            // maintain original spacing
            if replacement_len < original_len {
                result.push_str(&" ".repeat(original_len - replacement_len));
            }
        } else {
            result.push_str(token_text);
        }
    }

    result
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
}
