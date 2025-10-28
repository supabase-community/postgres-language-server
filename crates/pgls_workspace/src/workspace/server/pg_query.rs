use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::{Arc, LazyLock, Mutex};

use lru::LruCache;
use pgls_lexer::lex;
use pgls_query_ext::diagnostics::*;
use pgls_text_size::TextRange;
use regex::Regex;

use super::statement_identifier::StatementId;

const DEFAULT_CACHE_SIZE: usize = 1000;

pub struct PgQueryStore {
    ast_db: Mutex<LruCache<StatementId, Arc<Result<pgls_query::NodeEnum, SyntaxDiagnostic>>>>,
    plpgsql_db: Mutex<LruCache<StatementId, Result<(), SyntaxDiagnostic>>>,
}

impl PgQueryStore {
    pub fn new() -> PgQueryStore {
        PgQueryStore {
            ast_db: Mutex::new(LruCache::new(
                NonZeroUsize::new(DEFAULT_CACHE_SIZE).unwrap(),
            )),
            plpgsql_db: Mutex::new(LruCache::new(
                NonZeroUsize::new(DEFAULT_CACHE_SIZE).unwrap(),
            )),
        }
    }

    pub fn get_or_cache_ast(
        &self,
        statement: &StatementId,
    ) -> Arc<Result<pgls_query::NodeEnum, SyntaxDiagnostic>> {
        let mut cache = self.ast_db.lock().unwrap();

        if let Some(existing) = cache.get(statement) {
            return existing.clone();
        }

        let r = Arc::new(
            pgls_query::parse(&convert_to_positional_params(statement.content()))
                .map_err(SyntaxDiagnostic::from)
                .and_then(|ast| {
                    ast.into_root().ok_or_else(|| {
                        SyntaxDiagnostic::new("No root node found in parse result", None)
                    })
                }),
        );
        cache.put(statement.clone(), r.clone());
        r
    }

    pub fn get_or_cache_plpgsql_parse(
        &self,
        statement: &StatementId,
    ) -> Option<Result<(), SyntaxDiagnostic>> {
        let ast = self.get_or_cache_ast(statement);

        let create_fn = match ast.as_ref() {
            Ok(pgls_query::NodeEnum::CreateFunctionStmt(node)) => node,
            _ => return None,
        };

        let language = pgls_query_ext::utils::find_option_value(create_fn, "language")?;

        if language != "plpgsql" {
            return None;
        }

        let mut cache = self.plpgsql_db.lock().unwrap();

        if let Some(existing) = cache.get(statement) {
            return Some(existing.clone());
        }

        let sql_body = pgls_query_ext::utils::find_option_value(create_fn, "as")?;

        let start = statement.content().find(&sql_body)?;
        let end = start + sql_body.len();

        let range = TextRange::new(start.try_into().unwrap(), end.try_into().unwrap());

        let r = pgls_query::parse_plpgsql(statement.content())
            .or_else(|e| match &e {
                // ignore `is not a known variable` for composite types because libpg_query reports a false positive.
                // https://github.com/pganalyze/libpg_query/issues/159
                pgls_query::Error::Parse(err) if is_composite_type_error(err) => Ok(()),
                _ => Err(e),
            })
            .map_err(|e| SyntaxDiagnostic::new(e.to_string(), Some(range)));

        cache.put(statement.clone(), r.clone());

        Some(r)
    }
}

static COMPOSITE_TYPE_ERROR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"\\?"([^"\\]+\.[^"\\]+)\\?" is not a known variable"#).unwrap());

fn is_composite_type_error(err: &str) -> bool {
    COMPOSITE_TYPE_ERROR_RE.is_match(err)
}

// Keywords that, when preceding a named parameter, indicate that the parameter should be treated
// as an identifier rather than a positional parameter.
const IDENTIFIER_CONTEXT: [pgls_lexer::SyntaxKind; 15] = [
    pgls_lexer::SyntaxKind::TO_KW,
    pgls_lexer::SyntaxKind::FROM_KW,
    pgls_lexer::SyntaxKind::SCHEMA_KW,
    pgls_lexer::SyntaxKind::TABLE_KW,
    pgls_lexer::SyntaxKind::INDEX_KW,
    pgls_lexer::SyntaxKind::CONSTRAINT_KW,
    pgls_lexer::SyntaxKind::OWNER_KW,
    pgls_lexer::SyntaxKind::ROLE_KW,
    pgls_lexer::SyntaxKind::USER_KW,
    pgls_lexer::SyntaxKind::DATABASE_KW,
    pgls_lexer::SyntaxKind::TYPE_KW,
    pgls_lexer::SyntaxKind::CAST_KW,
    pgls_lexer::SyntaxKind::ALTER_KW,
    pgls_lexer::SyntaxKind::DROP_KW,
    // for schema.table style identifiers
    pgls_lexer::SyntaxKind::DOT,
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
        if kind == pgls_lexer::SyntaxKind::EOF {
            break;
        }

        let token_text = lexed.text(token_idx);

        if matches!(kind, pgls_lexer::SyntaxKind::NAMED_PARAM) {
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

    #[test]
    fn test_positional_params_in_grant() {
        let input = "grant usage on schema public, app_public, app_hidden to :DB_ROLE;";

        let result = convert_to_positional_params(input);

        assert_eq!(
            result,
            "grant usage on schema public, app_public, app_hidden to a       ;"
        );

        let store = PgQueryStore::new();

        let res = store.get_or_cache_ast(&StatementId::new(input));

        assert!(res.is_ok());
    }

    #[test]
    fn test_plpgsql_syntax_error() {
        let input = "
create function test_organisation_id ()
    returns setof text
    language plpgsql
    security invoker
    as $$
    -- syntax error here
    delare
        v_organisation_id uuid;
begin
    return next is(private.organisation_id(), v_organisation_id, 'should return organisation_id of token');
end
$$;
";

        let store = PgQueryStore::new();

        let res = store.get_or_cache_plpgsql_parse(&StatementId::new(input));

        assert!(matches!(res, Some(Err(_))));
    }

    #[test]
    fn test_plpgsql_valid() {
        let input = "
CREATE FUNCTION test_function()
    RETURNS integer
    LANGUAGE plpgsql
    AS $$
DECLARE
    counter integer := 0;
BEGIN
    counter := counter + 1;
    RETURN counter;
END;
$$;
";

        let store = PgQueryStore::new();

        let res = store.get_or_cache_plpgsql_parse(&StatementId::new(input));

        assert!(matches!(res, Some(Ok(_))));
    }

    #[test]
    fn test_non_plpgsql_function() {
        let input = "
CREATE FUNCTION add_numbers(a integer, b integer)
    RETURNS integer
    LANGUAGE sql
    AS $$
        SELECT a + b;
    $$;
";

        let store = PgQueryStore::new();

        let res = store.get_or_cache_plpgsql_parse(&StatementId::new(input));

        assert!(res.is_none());
    }

    #[test]
    fn test_non_function_statement() {
        let input = "SELECT * FROM users WHERE id = 1;";

        let store = PgQueryStore::new();

        let res = store.get_or_cache_plpgsql_parse(&StatementId::new(input));

        assert!(res.is_none());
    }

    #[test]
    fn test_cache_behavior() {
        let input = "
CREATE FUNCTION cached_function()
    RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    RAISE NOTICE 'Hello from cache test';
END;
$$;
";

        let store = PgQueryStore::new();
        let statement_id = StatementId::new(input);

        // First call should parse
        let res1 = store.get_or_cache_plpgsql_parse(&statement_id);
        assert!(matches!(res1, Some(Ok(_))));

        // Second call should return cached result
        let res2 = store.get_or_cache_plpgsql_parse(&statement_id);
        assert!(matches!(res2, Some(Ok(_))));
    }

    #[test]
    fn test_plpgsql_with_complex_body() {
        let input = "
CREATE FUNCTION complex_function(p_id integer)
    RETURNS TABLE(id integer, name text, status boolean)
    LANGUAGE plpgsql
    AS $$
DECLARE
    v_count integer;
    v_status boolean := true;
BEGIN
    SELECT COUNT(*) INTO v_count FROM users WHERE user_id = p_id;

    IF v_count > 0 THEN
        RETURN QUERY
        SELECT u.id, u.name, v_status
        FROM users u
        WHERE u.user_id = p_id;
    ELSE
        RAISE EXCEPTION 'User not found';
    END IF;
END;
$$;
";

        let store = PgQueryStore::new();

        let res = store.get_or_cache_plpgsql_parse(&StatementId::new(input));

        assert!(matches!(res, Some(Ok(_))));
    }

    #[test]
    fn test_invalid_ast() {
        let input = "CREATE FUNCTION invalid syntax here";

        let store = PgQueryStore::new();

        let res = store.get_or_cache_plpgsql_parse(&StatementId::new(input));

        assert!(res.is_none());
    }
}
