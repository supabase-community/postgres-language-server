use std::num::NonZeroUsize;
use std::sync::{Arc, LazyLock, Mutex};

use lru::LruCache;
pub use pgls_lexer::{convert_to_positional_params, convert_to_positional_params_with_metadata};
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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn parses_named_schema_qualifiers() {
        let cases = [
            r#"
SELECT
    customers.id
FROM staging.customers
CROSS JOIN :raw_data.migration_infos;
"#,
            "SELECT * FROM @schema.items;",
            "SELECT * FROM $schema.items;",
            r#"SELECT * FROM :"schema".items;"#,
            "SELECT * FROM :table;",
        ];

        let store = PgQueryStore::new();
        for input in cases {
            let result = store.get_or_cache_ast(&StatementId::new(input));
            assert!(result.is_ok(), "failed to parse {input:?}: {result:?}");
        }
    }

    #[test]
    fn preserves_array_slice_syntax_when_normalizing_parameters() {
        let input = "SELECT array_to_string(arr[3:array_upper(arr, 1)], ',') FROM t;";
        let normalized = convert_to_positional_params(input);

        assert_eq!(normalized, input);

        let store = PgQueryStore::new();
        let result = store.get_or_cache_ast(&StatementId::new(input));
        assert!(result.is_ok(), "failed to parse {input:?}: {result:?}");
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
