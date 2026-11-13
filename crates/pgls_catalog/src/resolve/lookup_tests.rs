//! The names that the resolver records as looked up through the search path, with and without a
//! database snapshot.

use super::{
    FunctionContext, FunctionParam, LookupKind, OperatorConstruct, ResolveParams, resolve,
    tests::database,
};
use crate::Catalog;

/// The lookups of `sql` after `setup`, each as its kind, the name, and the text of its span.
fn lookups_in(
    mut catalog: Catalog,
    setup: &str,
    sql: &str,
    function: Option<&FunctionContext>,
) -> Vec<String> {
    let search_path = vec!["public".to_owned()];
    if !setup.is_empty() {
        for stmt in pgls_query::parse(setup).expect("valid setup").stmts() {
            catalog.apply(stmt, &search_path);
        }
    }
    let stmt = pgls_query::parse(sql)
        .expect("valid sql")
        .into_root()
        .expect("a single statement");
    let resolution = resolve(ResolveParams {
        stmt: &stmt,
        catalog: &catalog,
        search_path: &search_path,
        function,
        sql: Some(sql),
        collect_references: false,
        collect_lookups: true,
    });
    let mut lookups = resolution.lookups;
    lookups.sort_by_key(|lookup| lookup.location);
    lookups
        .iter()
        .map(|lookup| {
            let kind = match lookup.kind {
                LookupKind::Relation => "relation".to_owned(),
                LookupKind::Column => "column".to_owned(),
                LookupKind::Type => "type".to_owned(),
                LookupKind::Function => "function".to_owned(),
                LookupKind::Operator(None) => "operator".to_owned(),
                LookupKind::Operator(Some(construct)) => {
                    format!("operator {}", construct.keywords())
                }
            };
            let span = lookup
                .span
                .map_or("?", |span| &sql[std::ops::Range::<usize>::from(span)]);
            format!("{kind} {} `{}`", lookup.name, span.trim())
        })
        .collect()
}

/// The lookups without a database snapshot.
fn lookups(sql: &str) -> Vec<String> {
    lookups_in(Catalog::new(None), "", sql, None)
}

/// The column lookups without a database snapshot.
fn columns(sql: &str) -> Vec<String> {
    only_columns(lookups(sql))
}

fn only_columns(lookups: Vec<String>) -> Vec<String> {
    lookups
        .into_iter()
        .filter(|lookup| lookup.starts_with("column "))
        .collect()
}

/// The lookups with the database of [`database`].
fn lookups_with_database(sql: &str) -> Vec<String> {
    lookups_in(Catalog::new(Some(database())), "", sql, None)
}

#[test]
fn records_unqualified_names_of_every_kind() {
    assert_eq!(
        lookups(
            "select abs(u.id)::text, \"Name\" from users u join public.posts p on u.id = p.user_id"
        ),
        [
            "function abs `abs`",
            "type text `text`",
            "column Name `\"Name\"`",
            "relation users `users`",
            "operator = `=`",
        ]
    );
}

#[test]
fn skips_qualified_names_and_sql_syntax() {
    assert!(
        lookups(
            "select pg_catalog.abs(1)::pg_catalog.text, 1::integer, extract(year from x), \
             coalesce(1, 2), 1 operator(pg_catalog.+) 2"
        )
        .is_empty()
    );
}

#[test]
fn cte_names_hide_relations_but_not_dml_targets() {
    assert_eq!(
        lookups("with a as (select * from b), b as (select * from a) select * from a, b"),
        ["relation b `b`"]
    );
    assert!(
        lookups("with recursive r as (select 1 union all select * from r) select * from r")
            .is_empty()
    );
    assert_eq!(
        lookups("with users as (select 1 as id) update users set id = 2 from users u"),
        ["relation users `users`"]
    );
    assert_eq!(
        lookups(
            "with users as (select 1 as id) merge into users t using users s on true \
             when matched then delete"
        ),
        ["relation users `users`"]
    );
}

#[test]
fn skips_select_into_locked_aliases_and_temporary_tables() {
    assert_eq!(
        lookups("select * into public.t from u for update of u"),
        ["relation u `u`"]
    );
    assert!(
        lookups_in(
            Catalog::new(None),
            "create temp table scratch (id int)",
            "select * from scratch",
            None
        )
        .is_empty()
    );
}

#[test]
fn records_relations_of_other_statements() {
    assert_eq!(
        lookups("truncate a, public.b, c"),
        ["relation a `a`", "relation c `c`"]
    );
    assert_eq!(lookups("lock table a"), ["relation a `a`"]);
    assert_eq!(lookups("copy a to stdout"), ["relation a `a`"]);
    assert_eq!(
        lookups("prepare p(text) as select * from a where a.x = $1"),
        ["type text `text`", "relation a `a`", "operator = `=`"]
    );
    assert_eq!(lookups("explain select * from a"), ["relation a `a`"]);
    assert_eq!(
        lookups("declare c cursor for select * from a"),
        ["relation a `a`"]
    );
    assert_eq!(
        lookups("create view public.v as select * from a"),
        ["relation a `a`"]
    );
    assert_eq!(
        lookups("create table public.t as select * from a"),
        ["relation a `a`"]
    );
}

#[test]
fn skips_the_relations_of_ddl() {
    assert!(lookups("create table t (id int references u)").is_empty());
    assert!(lookups("drop table t").is_empty());
}

#[test]
fn records_types_and_functions_of_ddl() {
    assert_eq!(
        lookups("create table public.t (a uuid default gen_random_uuid() check (a <> b))"),
        [
            "type uuid `uuid`",
            "function gen_random_uuid `gen_random_uuid`",
            "operator <> `<>`",
        ]
    );
    assert_eq!(
        lookups("alter table public.t alter column a type citext using a::citext"),
        ["type citext `citext`", "type citext `citext`"]
    );
    assert_eq!(
        lookups("drop function f(anyarray)"),
        ["type anyarray `anyarray`"]
    );
}

#[test]
fn skips_serial_only_in_column_definitions() {
    assert_eq!(
        lookups("create table public.t (a serial, b bigserial, c text)"),
        ["type text `text`"]
    );
    assert!(lookups("alter table public.t add column d serial8").is_empty());
    assert_eq!(
        lookups("alter table public.t alter column a type serial"),
        ["type serial `serial`"]
    );
    assert_eq!(lookups("select 1::serial"), ["type serial `serial`"]);
}

#[test]
fn records_the_signature_of_a_sql_function_but_not_its_body() {
    assert_eq!(
        lookups("create function public.f(x int default g(), y jsonb) returns text return abs(x)"),
        ["function g `g`", "type jsonb `jsonb`", "type text `text`"]
    );
}

#[test]
fn records_operator_constructs() {
    assert_eq!(
        lookups(
            "select t.a in (1), t.a not in (1), t.a in (select 1), t.a like 'x', \
             t.a not ilike 'x', t.a not similar to 'x', t.a is not distinct from t.b, \
             nullif(t.a, t.b), t.a not between symmetric 1 and 2, t.a = any (array[1]), \
             t.a < all (select 1) from t order by t.a using <"
        ),
        [
            "operator IN = `in`",
            "operator NOT IN <> `not in`",
            "operator IN = `in`",
            "operator LIKE ~~ `like`",
            "operator NOT ILIKE !~~* `not ilike`",
            "operator NOT SIMILAR TO !~ `not similar to`",
            "operator IS NOT DISTINCT FROM = `is not distinct from`",
            "operator NULLIF = `nullif`",
            "operator NOT BETWEEN < `not between symmetric`",
            "operator = `=`",
            "operator < `<`",
            "relation t `t`",
            "operator < `<`",
        ]
    );
}

#[test]
fn distinct_from_a_bare_null_is_a_null_test() {
    assert!(lookups("select 1 is distinct from null, null is not distinct from 1").is_empty());
    assert_eq!(
        lookups("select 1 is distinct from null::jsonb"),
        [
            "operator IS DISTINCT FROM = `is distinct from`",
            "type jsonb `jsonb`"
        ]
    );
}

#[test]
fn records_columns_only_in_queries_with_relations() {
    assert!(lookups("select a, b").is_empty());
    assert_eq!(
        lookups("select (select id) from public.users u"),
        ["column id `id`"]
    );
    assert_eq!(
        lookups("update public.users set name = upper(name) returning id"),
        [
            "function upper `upper`",
            "column name `name`",
            "column id `id`"
        ]
    );
}

#[test]
fn skips_whole_rows_output_columns_and_single_relation_clauses() {
    assert!(lookups("select u from public.users u").is_empty());
    assert!(lookups("select u.id as n from public.users u order by n").is_empty());
    assert!(
        lookups(
            "select u.id from public.users u union select p.id from public.posts p order by id"
        )
        .is_empty()
    );
    assert_eq!(
        columns(
            "insert into public.users (id, name) values (1, 'a') \
             on conflict (id) where active do update set name = excluded.name where active \
             returning id"
        ),
        ["column active `active`", "column id `id`"]
    );
}

#[test]
fn columns_take_precedence_with_the_columns_of_the_database() {
    // Without the columns of `users`, `name` in GROUP BY may be the output column.
    assert!(lookups("select u.id as name from public.users u group by name").is_empty());
    // With them, the column of FROM takes precedence.
    assert_eq!(
        lookups_with_database("select u.id as name from public.users u group by name"),
        ["column name `name`"]
    );
    // In ORDER BY, the output column takes precedence.
    assert!(
        lookups_with_database("select u.id as name from public.users u order by name").is_empty()
    );
    // A column takes precedence over a whole row.
    assert_eq!(
        lookups_with_database("select name from public.users name"),
        ["column name `name`"]
    );
}

#[test]
fn parameters_of_sql_functions_are_not_columns() {
    let function = FunctionContext {
        function_name: "f".into(),
        params: vec![FunctionParam {
            name: Some("id".into()),
            type_schema: None,
            type_name: "int8".into(),
            is_array: false,
        }],
    };
    let sql = "select u.name from public.users u where u.id = id";
    assert!(only_columns(lookups_in(Catalog::new(None), "", sql, Some(&function))).is_empty());
    // A column with the name takes precedence.
    let sql = "select id from public.users u";
    assert_eq!(
        lookups_in(Catalog::new(Some(database())), "", sql, Some(&function)),
        ["column id `id`"]
    );
}

#[test]
fn constructs_name_their_operators() {
    assert_eq!(OperatorConstruct::Between.operators(), [">=", "<="]);
    assert_eq!(OperatorConstruct::NotIn.keywords(), "NOT IN");
}
