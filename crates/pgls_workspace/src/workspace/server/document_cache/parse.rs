use std::sync::{Arc, LazyLock};

use parking_lot::Mutex;
use pgls_lexer::convert_to_positional_params;
use pgls_query_ext::{
    diagnostics::*,
    function_body::{EmbeddedStatement, Language, function_body_location, sql_function_body},
};
use regex::Regex;
use rustc_hash::FxHashMap;

use crate::workspace::StatementId;

/// The tree of a statement, or why it doesn't parse.
pub(crate) type ParseResult = Result<Arc<pgls_query::NodeEnum>, SyntaxDiagnostic>;

/// The parse results of the statements of a document. They are kept until the statement is
/// no longer in the document, see [`PgQueryStore::remove`].
pub(super) struct PgQueryStore {
    /// The tree is in an `Arc` of its own, so that the analyser can share it.
    ast_db: Mutex<FxHashMap<StatementId, Arc<ParseResult>>>,
    plpgsql_db: Mutex<FxHashMap<StatementId, Result<(), SyntaxDiagnostic>>>,
    /// The statements embedded in each top-level statement, like those of a SQL function body.
    embedded_db: Mutex<FxHashMap<StatementId, Arc<[EmbeddedStatement]>>>,
}

/// The results that [`PgQueryStore::take`] took out of a store, for the caller to drop.
#[derive(Default)]
pub(super) struct ParseEntries {
    ast_db: FxHashMap<StatementId, Arc<ParseResult>>,
    plpgsql_db: FxHashMap<StatementId, Result<(), SyntaxDiagnostic>>,
    embedded_db: FxHashMap<StatementId, Arc<[EmbeddedStatement]>>,
}

impl ParseEntries {
    pub(super) fn is_empty(&self) -> bool {
        self.ast_db.is_empty() && self.plpgsql_db.is_empty() && self.embedded_db.is_empty()
    }
}

impl PgQueryStore {
    pub(super) fn new() -> PgQueryStore {
        PgQueryStore {
            ast_db: Mutex::default(),
            plpgsql_db: Mutex::default(),
            embedded_db: Mutex::default(),
        }
    }

    /// The number of statements embedded in `statement`, if they are cached.
    pub(super) fn cached_embedded_len(&self, statement: &StatementId) -> Option<usize> {
        self.embedded_db
            .lock()
            .get(statement)
            .map(|embedded| embedded.len())
    }

    /// The number of statements with a cached parse result.
    pub(super) fn len(&self) -> usize {
        self.ast_db.lock().len()
    }

    /// Takes every cached result out, with the memory of the maps, for the caller to drop.
    pub(super) fn take(&self) -> ParseEntries {
        ParseEntries {
            ast_db: std::mem::take(&mut *self.ast_db.lock()),
            plpgsql_db: std::mem::take(&mut *self.plpgsql_db.lock()),
            embedded_db: std::mem::take(&mut *self.embedded_db.lock()),
        }
    }

    /// Drops the results of the top-level statement `statement` and of its embedded statements.
    /// Returns the ids of the embedded statements, which are cached only if those of
    /// `statement` are.
    pub(super) fn remove(&self, statement: &StatementId) -> Vec<StatementId> {
        debug_assert!(statement.is_root());
        let embedded: Vec<StatementId> = self
            .embedded_db
            .lock()
            .remove(statement)
            .iter()
            .flat_map(|embedded| embedded.iter())
            .map(|embedded| statement.create_child(&embedded.sql))
            .collect();
        let mut asts = self.ast_db.lock();
        let mut plpgsql = self.plpgsql_db.lock();
        for id in embedded.iter().chain([statement]) {
            asts.remove(id);
            plpgsql.remove(id);
        }
        embedded
    }

    /// Shrinks the maps that are less than a quarter full, see [`shrink`].
    pub(super) fn shrink(&self) {
        shrink(&mut self.ast_db.lock());
        shrink(&mut self.plpgsql_db.lock());
        shrink(&mut self.embedded_db.lock());
    }

    /// The number of entries that the maps have room for.
    #[cfg(test)]
    pub(super) fn capacity(&self) -> usize {
        self.ast_db.lock().capacity()
            + self.plpgsql_db.lock().capacity()
            + self.embedded_db.lock().capacity()
    }

    /// The statements embedded in the top-level statement `statement`, in order: the statements
    /// of the body of a SQL function. Empty if it doesn't parse. They are only cached if
    /// `may_insert` returns `true`, see [`get_or_cache_ast`](Self::get_or_cache_ast).
    pub(super) fn get_or_cache_embedded(
        &self,
        statement: &StatementId,
        may_insert: impl Fn() -> bool,
    ) -> Arc<[EmbeddedStatement]> {
        debug_assert!(statement.is_root());
        if let Some(existing) = self.embedded_db.lock().get(statement) {
            return Arc::clone(existing);
        }
        let embedded: Arc<[EmbeddedStatement]> =
            match self.get_or_cache_ast(statement, &may_insert).as_ref() {
                Ok(ast) => sql_function_body(ast, statement.content()).into(),
                Err(_) => Arc::default(),
            };
        let mut cache = self.embedded_db.lock();
        if may_insert() {
            cache.insert(statement.clone(), Arc::clone(&embedded));
        }
        embedded
    }

    /// The tree of `statement`, or why it doesn't parse. It is only cached if `may_insert`
    /// returns `true`, which is checked while the cache is locked, so that it can't be inserted
    /// after [`Self::remove`] ran for an edit that `may_insert` doesn't know of yet.
    pub(super) fn get_or_cache_ast(
        &self,
        statement: &StatementId,
        may_insert: impl Fn() -> bool,
    ) -> Arc<ParseResult> {
        let mut cache = self.ast_db.lock();

        if let Some(existing) = cache.get(statement) {
            return existing.clone();
        }

        let r = Arc::new(
            pgls_query::parse(&convert_to_positional_params(statement.content()))
                .map_err(SyntaxDiagnostic::from)
                .and_then(|ast| {
                    ast.into_root().map(Arc::new).ok_or_else(|| {
                        SyntaxDiagnostic::new("No root node found in parse result", None)
                    })
                }),
        );
        if may_insert() {
            cache.insert(statement.clone(), r.clone());
        }
        r
    }

    pub(super) fn get_or_cache_plpgsql_parse(
        &self,
        statement: &StatementId,
        may_insert: impl Fn() -> bool,
    ) -> Option<Result<(), SyntaxDiagnostic>> {
        let ast = self.get_or_cache_ast(statement, &may_insert);

        let create_fn = match ast.as_ref().as_deref() {
            Ok(pgls_query::NodeEnum::CreateFunctionStmt(node)) => node,
            _ => return None,
        };

        if Language::of(create_fn) != Some(Language::PlPgSql) {
            return None;
        }

        let mut cache = self.plpgsql_db.lock();

        if let Some(existing) = cache.get(statement) {
            return Some(existing.clone());
        }

        // The error has no location, so it covers the body, or the whole statement if the body
        // can't be found.
        let range = function_body_location(create_fn, statement.content()).map(|(range, _)| range);

        let r = pgls_query::parse_plpgsql(statement.content())
            .or_else(|e| match &e {
                // ignore `is not a known variable` for composite types because libpg_query reports a false positive.
                // https://github.com/pganalyze/libpg_query/issues/159
                pgls_query::Error::Parse(err) if is_composite_type_error(err) => Ok(()),
                pgls_query::Error::Parse(err)
                    if NO_CATALOG_ERRORS
                        .iter()
                        .any(|message| err.contains(message)) =>
                {
                    Ok(())
                }
                _ => Err(e),
            })
            .map_err(|e| SyntaxDiagnostic::new(e.to_string(), range));

        if may_insert() {
            cache.insert(statement.clone(), r.clone());
        }

        Some(r)
    }
}

/// Errors libpg_query reports on valid PL/pgSQL functions because it compiles them without a
/// catalog: it only knows the types of `pg_catalog`, reads an unknown type in `public` as
/// `record`, so an array of one as `_record`, finds no element type for a `VARIADIC` array, and
/// fails with `Not implemented` on anything else, e.g. a type in another schema.
/// https://github.com/pganalyze/libpg_query/blob/18.1.0/src/postgres/src_backend_utils_cache_syscache.c#L332-L344
const NO_CATALOG_ERRORS: &[&str] = &[
    "Not implemented",
    "cannot accept type _record",
    "cannot return type _record",
    "VARIADIC parameter must be an array",
];

#[expect(
    clippy::unwrap_used,
    reason = "the pattern is a constant, which `recognises_composite_type_errors` compiles"
)]
static COMPOSITE_TYPE_ERROR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"\\?"([^"\\]+\.[^"\\]+)\\?" is not a known variable"#).unwrap());

/// Shrinks `map` if it is less than a quarter full, so that a document that got much shorter
/// doesn't keep the memory of the statements it had.
pub(super) fn shrink<V>(map: &mut FxHashMap<StatementId, V>) {
    if map.capacity() > 4 * map.len().max(16) {
        map.shrink_to_fit();
    }
}

fn is_composite_type_error(err: &str) -> bool {
    COMPOSITE_TYPE_ERROR_RE.is_match(err)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dropping_results_releases_the_maps() {
        let store = PgQueryStore::new();
        let statements: Vec<StatementId> = (0..200)
            .map(|i| {
                StatementId::new(&format!(
                    "create function f{i}() returns int language sql as 'select {i}';"
                ))
            })
            .collect();
        let fill = || {
            for id in &statements {
                store.get_or_cache_embedded(id, || true);
                store.get_or_cache_plpgsql_parse(id, || true);
            }
        };

        fill();
        for id in &statements[1..] {
            store.remove(id);
        }
        store.shrink();
        assert_eq!(store.len(), 1);
        assert!(store.capacity() < 3 * 64, "{}", store.capacity());

        fill();
        assert!(!store.take().is_empty());
        assert_eq!(store.capacity(), 0);
    }

    #[test]
    fn test_positional_params_in_grant() {
        let input = "grant usage on schema public, app_public, app_hidden to :DB_ROLE;";

        let result = convert_to_positional_params(input);

        assert_eq!(
            result,
            "grant usage on schema public, app_public, app_hidden to db_role ;"
        );

        let store = PgQueryStore::new();

        let res = store.get_or_cache_ast(&StatementId::new(input), || true);

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
            let result = store.get_or_cache_ast(&StatementId::new(input), || true);
            assert!(result.is_ok(), "failed to parse {input:?}: {result:?}");
        }
    }

    #[test]
    fn preserves_array_slice_syntax_when_normalizing_parameters() {
        let input = "SELECT array_to_string(arr[3:array_upper(arr, 1)], ',') FROM t;";
        let normalized = convert_to_positional_params(input);

        assert_eq!(normalized, input);

        let store = PgQueryStore::new();
        let result = store.get_or_cache_ast(&StatementId::new(input), || true);
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

        let res = store.get_or_cache_plpgsql_parse(&StatementId::new(input), || true);

        assert!(matches!(res, Some(Err(_))));
    }

    #[test]
    fn plpgsql_syntax_errors_cover_quoted_bodies() {
        // A default with the same text as the body comes first, and the body has `''`.
        for (input, body) in [
            (
                "create function f(t text default 'delare x') returns void language plpgsql as 'delare x'",
                "'delare x'",
            ),
            (
                "create function f() returns void language plpgsql as 'begin raise notice ''x''; ende'",
                "'begin raise notice ''x''; ende'",
            ),
        ] {
            let store = PgQueryStore::new();
            let Some(Err(diagnostic)) =
                store.get_or_cache_plpgsql_parse(&StatementId::new(input), || true)
            else {
                panic!("expected a syntax error for {input}");
            };
            let span = pgls_diagnostics::Diagnostic::location(&diagnostic)
                .span
                .expect("a span");
            let start = input.rfind(body).unwrap() + 1;
            assert_eq!(
                std::ops::Range::<usize>::from(span),
                start..start + body.len() - 2,
                "{input}"
            );
        }
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

        let res = store.get_or_cache_plpgsql_parse(&StatementId::new(input), || true);

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

        let res = store.get_or_cache_plpgsql_parse(&StatementId::new(input), || true);

        assert!(res.is_none());
    }

    #[test]
    fn test_non_function_statement() {
        let input = "SELECT * FROM users WHERE id = 1;";

        let store = PgQueryStore::new();

        let res = store.get_or_cache_plpgsql_parse(&StatementId::new(input), || true);

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
        let res1 = store.get_or_cache_plpgsql_parse(&statement_id, || true);
        assert!(matches!(res1, Some(Ok(_))));

        // Second call should return cached result
        let res2 = store.get_or_cache_plpgsql_parse(&statement_id, || true);
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

        let res = store.get_or_cache_plpgsql_parse(&StatementId::new(input), || true);

        assert!(matches!(res, Some(Ok(_))));
    }

    #[test]
    fn test_invalid_ast() {
        let input = "CREATE FUNCTION invalid syntax here";

        let store = PgQueryStore::new();

        let res = store.get_or_cache_plpgsql_parse(&StatementId::new(input), || true);

        assert!(res.is_none());
    }

    #[test]
    fn plpgsql_valid_without_a_catalog() {
        for input in [
            "create function f(x my_enum[]) returns void language plpgsql as $$ begin end $$",
            "create function f() returns my_enum[] language plpgsql as $$ begin return null; end $$",
            "create function f() returns void language plpgsql as $$ declare u auth.users; begin end $$",
            "create function f(variadic b int[]) returns void language plpgsql as $$ begin end $$",
        ] {
            let store = PgQueryStore::new();
            let res = store.get_or_cache_plpgsql_parse(&StatementId::new(input), || true);
            assert!(matches!(res, Some(Ok(()))), "{input}: {res:?}");
        }
    }

    #[test]
    fn recognises_composite_type_errors() {
        assert!(is_composite_type_error(
            r#""v_row.id" is not a known variable"#
        ));
        assert!(!is_composite_type_error(
            r#""v_id" is not a known variable"#
        ));
    }

    #[test]
    fn a_panic_while_the_cache_is_locked_does_not_break_it() {
        let store = PgQueryStore::new();
        let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _cache = store.ast_db.lock();
            panic!("a request panicked while it held the cache");
        }));
        assert!(panicked.is_err());

        let res = store.get_or_cache_ast(&StatementId::new("select 1;"), || true);

        assert!(res.is_ok());
    }
}
