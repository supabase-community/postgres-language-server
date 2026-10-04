//! Compares the static analysis with Postgres. For each statement, Postgres either accepts it
//! (Parse/Describe without executing it, or executing it in a rolled-back transaction for
//! DDL) or rejects it with a SQLSTATE. The corpus states the expected outcome, which is
//! checked against Postgres first. Then:
//!
//! - statements Postgres accepts must not produce any finding, and every output type we infer
//!   must be the type Postgres describes;
//! - statements Postgres rejects with a type error must produce the matching finding, unless
//!   the case is marked as not detected yet.
//!
//! Every case runs twice: with the setup in the database snapshot, and with the setup applied
//! as statements of the file on top of the snapshot.

use std::sync::Arc;

use pgls_query::protobuf;
use sqlx::{Column, Executor, PgPool};

use super::{FindingKind, ResolveParams, query_output_types, resolve};
use crate::typing::{Type, TypeId};
use crate::{Catalog, CatalogBase, Snapshot};

const SETUP: &str = "
create type mood as enum ('happy', 'sad');
create domain posint as int check (value > 0);
create type pair as (a int, b text);
create table items (
    id int8 primary key, name text not null, price numeric, qty int4, tags text[],
    data jsonb, created_at timestamptz, mood mood, score posint, p pair, flag bool
);
create table logs (id serial, item_id int8, note varchar(20), at date);
create function add_one(x int) returns int language sql as 'select x + 1';
create function greet(name text, greeting text default 'hello') returns text language sql
    as 'select greeting || name';
create function total(variadic nums int[]) returns int language sql
    as 'select sum(n)::int from unnest(nums) n';
create function pick(a anyelement, b anyelement) returns anyelement language sql as 'select a';
create view item_names as select id, name from items;
";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Operator,
    Function,
    Cast,
    Assignment,
    Return,
}

#[derive(Debug, Clone, Copy)]
enum Expect {
    /// Postgres accepts the statement.
    Accept,
    /// Postgres rejects it with this SQLSTATE, and we report it.
    Detect(&'static str, Kind),
    /// Postgres rejects it with this SQLSTATE, but we don't detect it yet.
    Miss(&'static str),
}

use Expect::{Accept, Detect, Miss};

const CASES: &[(&str, Expect)] = &[
    // Operators
    (
        "select 1 + 1, 1 + 1.5, '1' + 1, -1, 1::int2 + 1::int8",
        Accept,
    ),
    (
        "select now() - interval '1 day', date '2020-01-01' + 1, now() > '2020-01-01'",
        Accept,
    ),
    ("select 'a' || 1, array[1] || 2, 1 = 1.5, 'a' = 'b'", Accept),
    (
        "select '{}'::jsonb -> 'a', '{}'::jsonb ->> 1, 'abc' ~ 'b'",
        Accept,
    ),
    ("select 1 operator(pg_catalog.+) 1", Accept),
    // Integer constants too large for `Ival`, typed by `make_const`
    (
        "select -2147483648, 2147483648, -9223372036854775808, 9223372036854775808",
        Accept,
    ),
    ("select '[]'::jsonb -> -2147483648", Accept),
    // Unknown-type output columns of subqueries and CTEs become text
    ("select val from (select null as val) s", Accept),
    ("with c as (select 'x' as v) select v from c", Accept),
    ("select (select 'x')", Accept),
    (
        "select array_agg(distinct val) from (select null as val from generate_series(1, 2)) s",
        Accept,
    ),
    (
        "select 'a' like 'b', 1 in (1, 2), 1 between 0 and 2, 1 is distinct from 2",
        Accept,
    ),
    ("select '1' + '1'", Detect("42725", Kind::Operator)),
    ("select 1 + now()", Detect("42883", Kind::Operator)),
    (
        "select 1 from items where name + 1 > 0",
        Detect("42883", Kind::Operator),
    ),
    (
        "select 1 from items i join logs l on i.name + 1 = l.id",
        Detect("42883", Kind::Operator),
    ),
    (
        "select 1 from items group by name + 1",
        Detect("42883", Kind::Operator),
    ),
    (
        "select 1 from items having name + 1 > 0",
        Detect("42883", Kind::Operator),
    ),
    (
        "select 1 from items order by name + 1",
        Detect("42883", Kind::Operator),
    ),
    (
        "delete from items where name + 1 > 0",
        Detect("42883", Kind::Operator),
    ),
    (
        "update items set qty = 1 where name + 1 > 0",
        Detect("42883", Kind::Operator),
    ),
    (
        "update items set qty = 1 returning name + 1",
        Detect("42883", Kind::Operator),
    ),
    (
        "select 1 in (1, 2), 1 = any(array[1, 2]), 1 between 0 and 2",
        Accept,
    ),
    ("select name as n from items group by n order by n", Accept),
    (
        "select name like 'x%' from items where flag and not false",
        Accept,
    ),
    (
        "select qty from items window w as (partition by flag order by qty)",
        Accept,
    ),
    (
        "select case qty when name then 1 else 2 end from items",
        Detect("42883", Kind::Operator),
    ),
    ("select 1 = any(array[1, 2])", Accept),
    (
        "select qty = any(array[name]) from items",
        Detect("42883", Kind::Operator),
    ),
    ("select 1 in (1, 2)", Accept),
    (
        "select 1 from items where qty + 1 > 0 and name like 'a%'",
        Accept,
    ),
    ("select q.q from generate_series(1, 2) q", Accept),
    (
        "select * from rows from (unnest(array[1, 2], array['a', 'b']), generate_series(1, 2)) with ordinality z(a, b, c, o)",
        Accept,
    ),
    (
        "select i, v from unnest(array[1, 2], array['a', 'b']) u(i, v)",
        Accept,
    ),
    (
        "select tsquery('a & b'), regtype('int4'), inet(text('127.0.0.1'))",
        Accept,
    ),
    (
        "select js from json_populate_record(null::pair, '{}') q(js)",
        Accept,
    ),
    (
        "select b from json_populate_record(null::pair, '{}') q",
        Accept,
    ),
    ("select jsonb_delete('{\"a\": 1}'::jsonb, 'a')", Accept),
    ("select nummultirange(numrange(1, 2))", Accept),
    ("select array[1] = array[1], array[1] = '{1}'", Accept),
    ("select name(i) from items i", Accept),
    (
        "create function f() returns boolean begin atomic ;;return false;; end",
        Accept,
    ),
    (
        "insert into items (id, name) values (1, 'a') on conflict (id) where name like 'a%' do nothing",
        Accept,
    ),
    ("select row(1, 2) = row(1, 2)", Accept),
    ("select i = i from items i", Accept),
    ("select i is distinct from i from items i", Accept),
    ("select (1, 'a')::pair = (1, 'a')::pair", Accept),
    ("select (1, 'a')::pair in ((1, 'a')::pair)", Accept),
    ("select i in (select i from items i) from items i", Accept),
    ("select qty in (select id from items) from items", Accept),
    ("select qty = any(select id from items) from items", Accept),
    ("select 1 = any('{1,2}')", Accept),
    ("select 'happy'::mood = 'happy'", Accept),
    ("select '[1,2)'::int4range = '[1,2)'", Accept),
    ("select array[1] = array[1]", Accept),
    ("select array[1] = '{1}'", Accept),
    (
        "select 1 in (qty, name) from items",
        Detect("42883", Kind::Operator),
    ),
    ("select 1 between 0 and 2", Accept),
    (
        "select name between qty and id from items",
        Detect("42883", Kind::Operator),
    ),
    (
        "select name is distinct from qty from items",
        Detect("42883", Kind::Operator),
    ),
    (
        "select nullif(qty, name) from items",
        Detect("42883", Kind::Operator),
    ),
    ("select 'a' like 'b%'", Accept),
    ("select 1 like 'b%'", Detect("42883", Kind::Operator)),
    ("select 'a' ilike 'b%'", Accept),
    ("select 'a' similar to 'b%'", Accept),
    (
        "select 1 from items window w as (partition by name + 1)",
        Detect("42883", Kind::Operator),
    ),
    ("select 1 from items where not flag", Accept),
    (
        "select name + 1 from items",
        Detect("42883", Kind::Operator),
    ),
    (
        "select mood + 1 from items",
        Detect("42883", Kind::Operator),
    ),
    // Functions
    (
        "select abs('1'), to_char(now(), 'YYYY'), round(1), round(1.5, 1)",
        Accept,
    ),
    (
        "select substr('abc', 1), left('abc', 1), concat(1, 'a'), format('%s', 1)",
        Accept,
    ),
    (
        "select make_interval(days => 1), generate_series(1, 10), jsonb_build_object('a', 1)",
        Accept,
    ),
    (
        "select array_append(array[1], 2), array_position(array[1], 1), array_cat(array[1], array[2.5])",
        Accept,
    ),
    (
        "select int4(1.5), pg_catalog.length('a'), date_trunc('day', now())",
        Accept,
    ),
    (
        "select extract(day from now()), now() at time zone 'utc'",
        Accept,
    ),
    (
        "select add_one(1), add_one('1'), greet('x'), greet('x', 'hi'), greet(name => 'x')",
        Accept,
    ),
    (
        "select total(1, 2, 3), pick(1, 2), pick('a'::text, 'b')",
        Accept,
    ),
    (
        "select count(*), sum(qty), array_agg(qty), max(name), avg(price) from items",
        Accept,
    ),
    (
        "select upper(name), length(name), lower(note) from items, logs",
        Accept,
    ),
    ("select length(1)", Detect("42883", Kind::Function)),
    ("select upper(1)", Detect("42883", Kind::Function)),
    (
        "select array_append(array[1], 'x'::text)",
        Detect("42883", Kind::Function),
    ),
    ("select add_one(now())", Detect("42883", Kind::Function)),
    ("select make_interval(nope => 1)", Miss("42883")),
    // Casts
    (
        "select 1::boolean, '1'::int, row(1, 2)::text, now()::date, 'happy'::mood",
        Accept,
    ),
    ("select now()::int", Detect("42846", Kind::Cast)),
    ("select true::date", Detect("42846", Kind::Cast)),
    // Common types
    (
        "select coalesce(1, 1.5), greatest(1, '2'), nullif(1, 2.5)",
        Accept,
    ),
    (
        "select case when flag then 1 else 2.5 end from items",
        Accept,
    ),
    ("select 1 union select 2::int8", Accept),
    ("values (1), (2.5)", Accept),
    ("select coalesce(1, 'a'::text)", Miss("42804")),
    // Relations and fields
    (
        "select (array[1,2])[1], (array[1,2])[1:2], tags[1] from items",
        Accept,
    ),
    ("select (p).a, (p).b from items", Accept),
    ("select * from item_names", Accept),
    (
        "select * from unnest(array[1,2]) with ordinality as u(x, n)",
        Accept,
    ),
    (
        "select id, name, price * qty, price * 2, qty + 1, name || '!', data -> 'k', \
         created_at - interval '1 hour', mood = 'happy', score + 1 from items",
        Accept,
    ),
    (
        "select i.id, l.note from items i join logs l on l.item_id = i.id \
         where i.price > 10 order by i.name",
        Accept,
    ),
    ("select current_date, current_user, localtimestamp", Accept),
    // Assignments
    (
        "insert into items (id, name, qty) values (1, 'a', 1)",
        Accept,
    ),
    (
        "insert into items (id, name, qty) values (1, 'a', '1'), (2, 'b', 2.5)",
        Accept,
    ),
    (
        "insert into items (id, name, data) values (1, 'a', '{}'), (2, 'b', null)",
        Accept,
    ),
    (
        "insert into items (id, name, tags, mood, price) values (1, 'a', array['x'], 'happy', 1)",
        Accept,
    ),
    (
        "insert into items (id, name, created_at, score) values (1, 'a', now(), 5)",
        Accept,
    ),
    (
        "insert into items (id, name, p) values (1, 'a', row(1, 'x'))",
        Accept,
    ),
    (
        "insert into items (id, name, tags[1]) values (1, 'a', 'x')",
        Accept,
    ),
    (
        "insert into items (id, name) select id, name from items",
        Accept,
    ),
    (
        "insert into items (id, name) select id, now() from items",
        Accept,
    ),
    (
        "insert into logs (item_id, note) values (1, 'x') returning id, note",
        Accept,
    ),
    (
        "update items set qty = qty + 1, name = 'b', price = 1, data = '{}'",
        Accept,
    ),
    ("update items set tags[1] = 'x', name = 1", Accept),
    (
        "insert into items (id, name) values (1, 'a') \
         on conflict (id) do update set qty = excluded.qty + 1",
        Accept,
    ),
    (
        "insert into items (id, name, qty) values (1, 'a', now())",
        Detect("42804", Kind::Assignment),
    ),
    (
        "insert into items (id, name, data) values (1, 'a', 'x'::text)",
        Detect("42804", Kind::Assignment),
    ),
    (
        "insert into items (id, qty) select id, now() from items",
        Detect("42804", Kind::Assignment),
    ),
    (
        "update items set qty = 'a'::text",
        Detect("42804", Kind::Assignment),
    ),
    (
        "update items set created_at = 1",
        Detect("42804", Kind::Assignment),
    ),
    // SQL function results
    (
        "create function f() returns int language sql as 'select 1'",
        Accept,
    ),
    (
        "create function f() returns int8 language sql as 'select 1'",
        Accept,
    ),
    (
        "create function f() returns items language sql as 'select * from items'",
        Accept,
    ),
    (
        "create function f() returns text language sql as 'select now()'",
        Accept,
    ),
    (
        "create function f() returns int language sql as 'select 1, 2'",
        Detect("42P13", Kind::Return),
    ),
    (
        "create function f() returns int language sql as 'select ''a''::text'",
        Detect("42P13", Kind::Return),
    ),
    (
        "create function f() returns int language sql as 'select 1::int8'",
        Accept,
    ),
    (
        "create function f() returns int language sql as 'select 1.5'",
        Accept,
    ),
    (
        "create function f() returns text language sql as 'select 1'",
        Accept,
    ),
    (
        "create function f() returns text language sql as 'select ''a'''",
        Accept,
    ),
    (
        "create function f() returns posint language sql as 'select 1'",
        Accept,
    ),
    (
        "create function f() returns pair language sql as 'select 1, ''a'''",
        Accept,
    ),
    (
        "create function f() returns pair language sql as 'select row(1, ''a'')::pair'",
        Accept,
    ),
    (
        "create function f() returns setof int language sql as 'select 1 union select 2'",
        Accept,
    ),
    (
        "create function f() returns int language sql as 'select ''1'''",
        Detect("42P13", Kind::Return),
    ),
    (
        "create function f() returns jsonb language sql as 'select ''{}'''",
        Detect("42P13", Kind::Return),
    ),
    (
        "create function f() returns jsonb language sql as 'select null'",
        Detect("42P13", Kind::Return),
    ),
    (
        "create function f() returns setof jsonb language sql as 'values (''{}'')'",
        Detect("42P13", Kind::Return),
    ),
    (
        "create function f() returns int language sql as 'select ''a'' union select ''b'''",
        Detect("42P13", Kind::Return),
    ),
    (
        "create function f() returns int language sql begin atomic select '1'; end",
        Detect("42P13", Kind::Return),
    ),
    (
        "create function f() returns mood language sql as 'select 1, 2'",
        Detect("42P13", Kind::Return),
    ),
    (
        "create function f() returns mood language sql as 'select ''happy'''",
        Detect("42P13", Kind::Return),
    ),
    (
        "create function f() returns pair language sql as 'select ''a''::text, 1'",
        Detect("42P13", Kind::Return),
    ),
    (
        "create function f() returns pair language sql as 'select now(), 1'",
        Detect("42P13", Kind::Return),
    ),
    (
        "create function f() returns pair language sql as 'select 1'",
        Detect("42P13", Kind::Return),
    ),
    (
        "create function f(out a int, out b text) language sql as 'select now(), ''x'''",
        Detect("42P13", Kind::Return),
    ),
    (
        "create function f() returns table (a int, b jsonb) language sql as 'select 1, ''{}'''",
        Detect("42P13", Kind::Return),
    ),
    (
        "create function f() returns items language sql as 'select id, name from items'",
        Detect("42P13", Kind::Return),
    ),
];

/// What Postgres says: the output column types, or the SQLSTATE of the error.
async fn postgres(pool: &PgPool, sql: &str) -> Result<Vec<Option<u32>>, String> {
    let code = |error: sqlx::Error| match error {
        sqlx::Error::Database(error) => error
            .code()
            .map(|code| code.into_owned())
            .unwrap_or_default(),
        error => panic!("{sql}: {error}"),
    };
    if sql.starts_with("create") {
        let mut transaction = pool.begin().await.unwrap();
        let result = transaction
            .execute(sql)
            .await
            .map(|_| Vec::new())
            .map_err(code);
        transaction.rollback().await.unwrap();
        return result;
    }
    let describe = pool.describe(sql).await.map_err(code)?;
    Ok(describe
        .columns()
        .iter()
        .map(|column| column.type_info().oid().map(|oid| oid.0))
        .collect())
}

/// What we say: the type findings, and the output column types if known.
fn analyse(catalog: &Catalog, sql: &str) -> (Vec<FindingKind>, Option<Vec<Option<Type>>>) {
    let search_path = vec!["public".to_owned()];
    let root = pgls_query::parse(sql)
        .unwrap()
        .into_root()
        .expect("one statement");
    let resolution = resolve(ResolveParams {
        stmt: &root,
        catalog,
        search_path: &search_path,
        function: None,
        sql: Some(sql),
    });
    let node = protobuf::Node { node: Some(root) };
    let types = query_output_types(&node, catalog, &search_path)
        .map(|columns| columns.into_iter().map(|column| column.ty).collect());
    let findings = resolution.findings.into_iter().map(|f| f.kind).collect();
    (findings, types)
}

fn kind(finding: &FindingKind) -> Option<Kind> {
    match finding {
        FindingKind::OperatorMismatch { .. } => Some(Kind::Operator),
        FindingKind::FunctionArgumentMismatch { .. } => Some(Kind::Function),
        FindingKind::InvalidCast { .. } => Some(Kind::Cast),
        FindingKind::AssignmentMismatch { .. } => Some(Kind::Assignment),
        FindingKind::FunctionReturnMismatch { .. } => Some(Kind::Return),
        _ => None,
    }
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn agrees_with_postgres(pool: PgPool) {
    let before = Arc::new(CatalogBase::new(Arc::new(
        Snapshot::load(&pool).await.unwrap(),
    )));
    pool.execute(SETUP).await.unwrap();
    let after = Arc::new(CatalogBase::new(Arc::new(
        Snapshot::load(&pool).await.unwrap(),
    )));

    let search_path = vec!["public".to_owned()];
    let in_snapshot = Catalog::new(Some(after));
    let mut in_file = Catalog::new(Some(before));
    for stmt in pgls_query::parse(SETUP).unwrap().stmts() {
        in_file.apply(stmt, &search_path);
    }

    let mut failures = Vec::new();
    let (mut typed, mut columns) = (0, 0);
    for (sql, expect) in CASES {
        let outcome = postgres(&pool, sql).await;
        match (expect, &outcome) {
            (Accept, Ok(_)) => {}
            (Detect(code, _) | Miss(code), Err(actual)) if code == actual => {}
            _ => {
                failures.push(format!(
                    "{sql}\n    postgres: {outcome:?}, expected {expect:?}"
                ));
                continue;
            }
        }
        for (mode, catalog) in [("snapshot", &in_snapshot), ("file", &in_file)] {
            let (findings, types) = analyse(catalog, sql);
            match (expect, &outcome) {
                (Accept, Ok(described)) => {
                    if !findings.is_empty() {
                        failures.push(format!("{sql}\n    [{mode}] false positive: {findings:?}"));
                    }
                    let Some(types) = types else { continue };
                    if types.len() != described.len() {
                        if !described.is_empty() {
                            failures.push(format!(
                                "{sql}\n    [{mode}] {} columns, postgres has {}",
                                types.len(),
                                described.len()
                            ));
                        }
                        continue;
                    }
                    for (position, (ty, oid)) in types.iter().zip(described).enumerate() {
                        columns += 1;
                        let Some(ty) = ty else { continue };
                        typed += 1;
                        let matches = match (ty, oid) {
                            // Postgres describes a top-level unknown literal as text.
                            (Type::UnknownLiteral, Some(25)) => true,
                            (Type::Named(TypeId::Snapshot(id)), Some(oid)) => {
                                *id == i64::from(*oid)
                            }
                            // File-created types have no oid to compare with.
                            (Type::Named(TypeId::File(_)), _) => true,
                            _ => false,
                        };
                        if !matches {
                            failures.push(format!(
                                "{sql}\n    [{mode}] column {position}: {ty:?}, postgres has oid {oid:?}"
                            ));
                        }
                    }
                }
                (Detect(_, expected), _) => {
                    if !findings.iter().any(|f| kind(f) == Some(*expected)) {
                        failures.push(format!("{sql}\n    [{mode}] not detected: {findings:?}"));
                    }
                }
                (Miss(_), _) if findings.iter().any(|f| kind(f).is_some()) => {
                    failures.push(format!("{sql}\n    [{mode}] now detected: {findings:?}"));
                }
                _ => {}
            }
        }
    }
    eprintln!("typed {typed} of {columns} output columns");
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}
