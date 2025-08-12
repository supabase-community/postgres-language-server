use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

use lru::LruCache;
use pgt_query_ext::diagnostics::*;
use pgt_text_size::TextRange;
use pgt_tokenizer::tokenize;

use super::statement_identifier::StatementId;

const DEFAULT_CACHE_SIZE: usize = 1000;

pub struct PgQueryStore {
    ast_db: Mutex<LruCache<StatementId, Arc<Result<pgt_query::NodeEnum, SyntaxDiagnostic>>>>,
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
    ) -> Arc<Result<pgt_query::NodeEnum, SyntaxDiagnostic>> {
        let mut cache = self.ast_db.lock().unwrap();

        if let Some(existing) = cache.get(statement) {
            return existing.clone();
        }

        let r = Arc::new(
            pgt_query::parse(&convert_to_positional_params(statement.content()))
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
            Ok(pgt_query::NodeEnum::CreateFunctionStmt(node)) => node,
            _ => return None,
        };

        let language = pgt_query_ext::utils::find_option_value(create_fn, "language")?;

        if language != "plpgsql" {
            return None;
        }

        let mut cache = self.plpgsql_db.lock().unwrap();

        if let Some(existing) = cache.get(statement) {
            return Some(existing.clone());
        }

        let sql_body = pgt_query_ext::utils::find_option_value(create_fn, "as")?;

        let start = statement.content().find(&sql_body)?;
        let end = start + sql_body.len();

        let range = TextRange::new(start.try_into().unwrap(), end.try_into().unwrap());

        let r = pgt_query::parse_plpgsql(statement.content())
            .map_err(|err| SyntaxDiagnostic::new(err.to_string(), Some(range)));
        cache.put(statement.clone(), r.clone());

        Some(r)
    }
}

/// Converts named parameters in a SQL query string to positional parameters.
///
/// This function scans the input SQL string for named parameters (e.g., `@param`, `:param`, `:'param'`)
/// and replaces them with positional parameters (e.g., `$1`, `$2`, etc.).
///
/// It maintains the original spacing of the named parameters in the output string.
///
/// Useful for preparing SQL queries for parsing or execution where named paramters are not supported.
pub fn convert_to_positional_params(text: &str) -> String {
    let mut result = String::new();
    let mut param_index = 1;
    let mut position = 0;

    for token in tokenize(text) {
        let token_len = token.len as usize;
        let token_text = &text[position..position + token_len];

        if matches!(token.kind, pgt_tokenizer::TokenKind::NamedParam { .. }) {
            let replacement = format!("${}", param_index);
            let original_len = token_text.len();
            let replacement_len = replacement.len();

            result.push_str(&replacement);

            // maintain original spacing
            if replacement_len < original_len {
                result.push_str(&" ".repeat(original_len - replacement_len));
            }

            param_index += 1;
        } else {
            result.push_str(token_text);
        }

        position += token_len;
    }

    result
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
            "select * from users where id = $1 and name = $2 and email = $3;"
        );
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
