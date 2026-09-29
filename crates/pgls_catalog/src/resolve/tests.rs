use std::sync::Arc;

use pgls_schema_cache::{Function, FunctionArg, FunctionArgs, PostgresType, ProcKind, SchemaCache};

use super::*;
use crate::{Catalog, CatalogBase};

fn schema(name: &str) -> pgls_schema_cache::Schema {
    pgls_schema_cache::Schema {
        name: name.into(),
        ..Default::default()
    }
}

fn type_(id: i64, schema: &str, name: &str, attributes: &[(&str, i64)]) -> PostgresType {
    PostgresType {
        id,
        schema: schema.into(),
        name: name.into(),
        attributes: pgls_schema_cache::TypeAttributes {
            attrs: attributes
                .iter()
                .map(|(name, type_id)| pgls_schema_cache::PostgresTypeAttribute {
                    name: (*name).into(),
                    type_id: *type_id,
                })
                .collect(),
        },
        ..Default::default()
    }
}

fn function(name: &str, kind: ProcKind, modes: &[&str], defaults: usize) -> Function {
    let args = modes
        .iter()
        .enumerate()
        .map(|(i, mode)| FunctionArg {
            mode: (*mode).into(),
            name: String::new(),
            type_id: 25,
            has_default: Some(i + defaults >= modes.len()),
        })
        .collect();
    Function {
        schema: "pg_catalog".into(),
        name: name.into(),
        kind,
        args: FunctionArgs { args },
        ..Default::default()
    }
}

/// A database with a few builtins, the tables `users (id, name)` and `posts (id, user_id,
/// title)`, and the composite type `address (street, city)`.
fn database() -> Arc<CatalogBase> {
    const INT8: i64 = 20;
    const TEXT: i64 = 25;
    let mut types: Vec<PostgresType> = [
        (16, "bool"),
        (INT8, "int8"),
        (23, "int4"),
        (TEXT, "text"),
        (701, "float8"),
        (1043, "varchar"),
        (1082, "date"),
        (1184, "timestamptz"),
        (1700, "numeric"),
        (2205, "regclass"),
        (2249, "record"),
    ]
    .into_iter()
    .map(|(id, name)| type_(id, "pg_catalog", name, &[]))
    .collect();
    types.push(type_(
        9001,
        "public",
        "users",
        &[("id", INT8), ("name", TEXT)],
    ));
    types.push(type_(
        9002,
        "public",
        "posts",
        &[("id", INT8), ("user_id", INT8), ("title", TEXT)],
    ));
    types.push(type_(
        9003,
        "public",
        "address",
        &[("street", TEXT), ("city", TEXT)],
    ));

    let tables = [(1, "users"), (2, "posts")]
        .into_iter()
        .map(|(id, name)| pgls_schema_cache::Table {
            id,
            schema: "public".into(),
            name: name.into(),
            ..Default::default()
        })
        .collect();

    let functions = vec![
        function("count", ProcKind::Aggregate, &[], 0),
        function("count", ProcKind::Aggregate, &["in"], 0),
        function("upper", ProcKind::Function, &["in"], 0),
        function("lower", ProcKind::Function, &["in"], 0),
        function("max", ProcKind::Aggregate, &["in"], 0),
        function("now", ProcKind::Function, &[], 0),
        function("generate_series", ProcKind::Function, &["in", "in"], 0),
        function(
            "generate_series",
            ProcKind::Function,
            &["in", "in", "in"],
            0,
        ),
        function("concat", ProcKind::Function, &["variadic"], 0),
        function("round", ProcKind::Function, &["in", "in"], 1),
    ];

    let cache = SchemaCache {
        schemas: vec![schema("public"), schema("pg_catalog")],
        tables,
        types,
        functions,
        ..Default::default()
    };
    Arc::new(CatalogBase::new(Arc::new(cache)))
}

const SEARCH_PATH: &[&str] = &["public"];

fn search_path() -> Vec<String> {
    SEARCH_PATH.iter().map(|s| s.to_string()).collect()
}

/// Applies `setup` to the catalog, then resolves `sql`.
fn resolve_after(setup: &str, sql: &str, function: Option<&FunctionContext>) -> Resolution {
    let search_path = search_path();
    let mut catalog = Catalog::new(Some(database()));
    if !setup.is_empty() {
        for stmt in pgls_query::parse(setup).expect("valid setup").stmts() {
            catalog.apply(stmt, &search_path);
        }
    }
    let stmt = pgls_query::parse(sql)
        .expect("valid sql")
        .into_root()
        .expect("a single statement");
    resolve(ResolveParams {
        stmt: &stmt,
        catalog: &catalog,
        search_path: &search_path,
        function,
        sql: Some(sql),
    })
}

fn findings(sql: &str) -> Vec<FindingKind> {
    findings_after("", sql)
}

fn findings_after(setup: &str, sql: &str) -> Vec<FindingKind> {
    resolve_after(setup, sql, None)
        .findings
        .into_iter()
        .map(|finding| finding.kind)
        .collect()
}

#[track_caller]
fn assert_valid(sql: &str) {
    assert_eq!(findings(sql), [], "{sql}");
}

fn unknown_column(relation: Option<&str>, column: &str) -> FindingKind {
    FindingKind::UnknownColumn {
        relation: relation.map(str::to_owned),
        column: column.into(),
    }
}

#[test]
fn relations_and_schemas() {
    assert_eq!(
        findings("select * from nope"),
        [FindingKind::UnknownRelation {
            schema: None,
            name: "nope".into()
        }]
    );
    assert_eq!(
        findings("select * from nope.users"),
        [FindingKind::UnknownSchema {
            name: "nope".into()
        }]
    );
    assert_valid("select * from public.users");
}

#[test]
fn spans_cover_the_reference() {
    let resolution = resolve_after("", "select * from public.nope", None);
    let span = resolution.findings[0].span.unwrap();
    assert_eq!(
        &"select * from public.nope"[usize::from(span.start())..usize::from(span.end())],
        "public.nope"
    );
}

#[test]
fn columns() {
    assert_eq!(
        findings("select email from users"),
        [unknown_column(None, "email")]
    );
    assert_eq!(
        findings("select u.email from users u"),
        [unknown_column(Some("u"), "email")]
    );
    assert_eq!(
        findings("select users.id from users u"),
        [FindingKind::MissingFromClauseEntry {
            name: "users".into()
        }]
    );
    assert_eq!(
        findings("select public.users.nope from users"),
        [unknown_column(Some("public.users"), "nope")]
    );
    assert_valid("select id, name, users.id, public.users.id from users");
    assert_valid("select ctid, xmin, u.tableoid from users u");
    // Whole-row references and functional notation.
    assert_valid("select u, count(u), name(u) from users u");
    // Aliased column lists.
    assert_valid("select u.a, u.name from users as u(a)");
}

#[test]
fn ambiguity() {
    assert_eq!(
        findings("select id from users join posts on posts.user_id = users.id"),
        [FindingKind::AmbiguousColumn {
            column: "id".into(),
            candidates: vec!["users".into(), "posts".into()],
        }]
    );
    assert_valid("select id from users join posts using (id)");
    assert_valid("select id from users natural join posts");
    assert_valid("select j.id from users join posts using (id) as j");
    // The inner level shadows the outer one.
    assert_valid("select (select id from posts limit 1) from users");
    // Non-lateral subqueries don't see their siblings.
    assert_valid("select * from users, (select id from posts) as s");
}

#[test]
fn scopes() {
    assert_valid("select s.x from users cross join lateral (select users.id as x) as s");
    assert_valid(
        "select 1 from users o where exists (select 1 from posts a join posts b on a.id = o.id)",
    );
    assert_valid(
        "select * from users u where u.id in (select user_id from posts where posts.user_id = u.id)",
    );
    assert_eq!(
        findings("select * from users where exists (select 1 from posts where nope = 1)"),
        [unknown_column(None, "nope")]
    );
    // Output names are visible to ORDER BY and GROUP BY, but not to WHERE.
    assert_valid("select id as x from users order by x");
    assert_valid("select id as x from users group by rollup(x)");
    assert_valid("select upper(name) from users order by upper");
    assert_eq!(
        findings("select id as x from users where x = 1"),
        [unknown_column(None, "x")]
    );
    // Subquery columns.
    assert_valid(
        "select q.id, q.upper, q.x from (select users.*, upper(name), 1 as x from users) q",
    );
    assert_eq!(
        findings("select q.nope from (select id from users) q"),
        [unknown_column(Some("q"), "nope")]
    );
    // Subqueries in unusual places.
    assert_valid("select greatest((select id from users limit 1), 0)");
    assert_valid("select 1 from users order by (select max(id) from posts)");
}

#[test]
fn ctes() {
    assert_valid("with c as (select 1 as id) select id from c");
    assert_valid("with c(a) as (select 1 as x, 2 as y) select a, y from c");
    assert_eq!(
        findings("with c as (select 1 as id) select nope from c"),
        [unknown_column(None, "nope")]
    );
    assert_valid("with c as (select 1 as id) insert into users select id, 'x' from c");
    assert_valid("with c as (select 1 as id) update users set id = c.id from c");
    assert_valid("with c as (select 1 as id) delete from users using c where users.id = c.id");
    assert_valid("with d as (delete from users returning id) select id from d");
    assert_valid(
        "select * from (with recursive r(n) as (select 1 union all select n + 1 from r where n < 2) select n from r) as q",
    );
    assert_valid(
        "with recursive c(n) as (select 1 union all select n + 1 from c where n < 2) search depth first by n set ord select ord from c",
    );
}

#[test]
fn functions() {
    assert_eq!(
        findings("select nope()"),
        [FindingKind::UnknownFunction {
            schema: None,
            name: "nope".into(),
            arg_count: 0,
            name_exists: false
        }]
    );
    assert_eq!(
        findings("select upper('a', 'b')"),
        [FindingKind::UnknownFunction {
            schema: None,
            name: "upper".into(),
            arg_count: 2,
            name_exists: true
        }]
    );
    assert_valid("select count(*), count(id) from users");
    assert_valid("select concat('a', 'b', 'c'), round(1), round(1, 2)");
    assert_valid("select extract(year from now()), trim(' x '), substring('x' from 1)");
    assert_valid("select now() at time zone 'utc', position('a' in 'abc')");
    assert_valid("select coalesce(1, 2), nullif(1, 2), greatest(1, 2), current_date");
    assert_valid("select g from generate_series(1, 3) g");
    assert_valid("select r.ordinality from generate_series(1, 2) with ordinality as r");
    assert_valid("select generate_series.generate_series from generate_series(1, 2)");
}

#[test]
fn types() {
    assert_valid(
        "select 1::int, 'x'::varchar(10), now()::timestamp with time zone, 1::double precision, '{}'::int[], 1::numeric(10, 2)",
    );
    assert_valid("select null::address, null::users");
    assert_eq!(
        findings("select null::nope"),
        [FindingKind::UnknownType {
            schema: None,
            name: "nope".into()
        }]
    );
}

#[test]
fn inserts() {
    assert_valid("insert into users (id) values (1)");
    assert_valid("insert into users (id) values (default)");
    assert_valid("insert into users (id) overriding system value values (1)");
    assert_valid("insert into users values (1)");
    assert_valid("insert into users select * from users");
    assert_valid("insert into users (id, name) select id, name from users union select 1, 'a'");
    assert_valid(
        "insert into users values (1, 'x') on conflict (id) do update set name = excluded.name",
    );
    assert_valid("insert into users as u values (1, 'x') returning u.id, name");
    assert_eq!(
        findings("insert into users (id, name) values (1)"),
        [FindingKind::InsertColumnMismatch {
            expected: 2,
            found: 1
        }]
    );
    assert_eq!(
        findings("insert into users values (1, 'a', 'b')"),
        [FindingKind::InsertColumnMismatch {
            expected: 2,
            found: 3
        }]
    );
    assert_eq!(
        findings("insert into users (id, nope) values (1, 2)"),
        [unknown_column(Some("users"), "nope")]
    );
}

#[test]
fn updates_and_deletes() {
    assert_valid(
        "update users set name = posts.title from posts where posts.user_id = users.id returning users.*",
    );
    assert_valid("update users set (id, name) = (select id, title from posts limit 1)");
    assert_eq!(
        findings("update users set nope = 1"),
        [unknown_column(Some("users"), "nope")]
    );
    assert_eq!(
        findings("delete from users where nope = 1"),
        [unknown_column(None, "nope")]
    );
}

#[test]
fn wrappers() {
    assert_eq!(
        findings("create view v as select nope from users"),
        [unknown_column(None, "nope")]
    );
    assert_eq!(
        findings("create table t as select nope from users"),
        [unknown_column(None, "nope")]
    );
    assert_eq!(
        findings("explain select * from nope"),
        [FindingKind::UnknownRelation {
            schema: None,
            name: "nope".into()
        }]
    );
    assert_valid("copy (select id from users) to stdout");
}

#[test]
fn objects_created_in_the_file() {
    let setup = "create temp table scratch (id int);
                 create table items (id int, label text);
                 alter table items add column price numeric;
                 create function add_one(value int) returns int language sql as 'select value + 1';
                 create type mood as enum ('ok');";
    assert_eq!(
        findings_after(
            setup,
            "select scratch.id, items.price, add_one(1), null::mood from scratch, items"
        ),
        []
    );
    assert_eq!(
        findings_after(setup, "select add_one(1, 2)"),
        [FindingKind::UnknownFunction {
            schema: None,
            name: "add_one".into(),
            arg_count: 2,
            name_exists: true
        }]
    );
}

#[test]
fn unknown_objects_are_silent() {
    let setup = "do $$ begin execute 'create table x ()'; end $$;";
    assert_eq!(findings_after(setup, "select a.b, c from x as a, nope"), []);
    assert_eq!(findings_after(setup, "select anything()"), []);
    // XMLTABLE items are not modelled.
    assert_valid(
        "select t.anything from xmltable('/a' passing '<a/>' columns x int path 'x') as t",
    );
}

#[test]
fn function_bodies() {
    let function = FunctionContext {
        function_name: "get_name".into(),
        params: vec![
            FunctionParam {
                name: Some("row_arg".into()),
                type_schema: Some("public".into()),
                type_name: "users".into(),
                is_array: false,
            },
            FunctionParam {
                name: Some("prefix".into()),
                type_schema: None,
                type_name: "text".into(),
                is_array: false,
            },
        ],
    };
    let findings = |sql: &str| -> Vec<FindingKind> {
        resolve_after("", sql, Some(&function))
            .findings
            .into_iter()
            .map(|finding| finding.kind)
            .collect()
    };

    // GitHub issue #705.
    assert_eq!(findings("select row_arg.name"), []);
    assert_eq!(
        findings("select row_arg, prefix, get_name.prefix, $1, $2"),
        []
    );
    assert_eq!(findings("select prefix || name from users"), []);
    assert_eq!(
        findings("select row_arg.nope"),
        [unknown_column(Some("row_arg"), "nope")]
    );
    assert!(!resolve_after("", "select row_arg.name", Some(&function)).database_only);
}

#[test]
fn database_only() {
    let database_only = |setup: &str, sql: &str| resolve_after(setup, sql, None).database_only;

    assert!(database_only("", "select * from users"));
    assert!(database_only("", "select 1"));
    assert!(database_only(
        "",
        "insert into users (id) select id from posts"
    ));
    assert!(database_only("", "create view v as select id from users"));
    assert!(database_only(
        "",
        "select count(*) from users join posts using (id)"
    ));

    assert!(!database_only(
        "create table t2 (id int)",
        "select * from t2"
    ));
    assert!(!database_only(
        "alter table users add column x int",
        "select * from users"
    ));
    assert!(!database_only(
        "create table t (id int)",
        "with t as (select id from t) select id from t"
    ));
    assert!(!database_only("", "select * from nope"));
    assert!(!database_only("", "select 'users'::regclass"));
    assert!(!database_only("", "explain select 1"));
    assert!(!database_only("", "select id into t3 from users"));
    assert!(!database_only("", "select xmlelement(name a)"));
    assert!(!database_only("", "create table t4 (id int)"));
    assert!(!database_only(
        "do $$ begin end $$",
        "select * from anything"
    ));
}
