use std::sync::Arc;

use sqlx::{Executor, PgPool};

use super::{ColumnSource, FunctionContext, FunctionParam, Reference, ResolveParams, resolve};
use crate::{
    Catalog, CatalogBase, Snapshot,
    typing::{Type, format_type},
};

const SETUP: &str = r#"
    create table users (id int8 primary key, name text);
    create table posts (id int8 primary key, user_id int8, title text);
    create function pick(i int) returns int language sql as 'select i';
    create function pick(t text) returns text language sql as 'select t';
    create function sum_all(base int, variadic nums int[]) returns int
        language sql as 'select base';
    create function greet(name text, greeting text default 'hi') returns text
        language sql as 'select greeting || name';
    create function amb(a int, b text) returns int language sql as 'select 1';
    create function amb(c text, d int) returns int language sql as 'select 1';
    create type mood as enum ('happy', 'sad');
    create table people (id int8, current mood);
    create function cheer(m mood) returns mood language sql as 'select m';
    create function cheer_all(variadic moods mood[]) returns int language sql as 'select 1';
    create procedure do_it(a int, b text) language sql as 'select 1';
    create procedure with_out(a int, out b int) language plpgsql as $$ begin b := a; end $$;
    create procedure with_inout(a int, out b int, inout c text) language plpgsql
        as $$ begin b := a; end $$;
"#;

async fn catalog(test_db: &PgPool) -> (Catalog, Vec<String>) {
    test_db.execute(SETUP).await.expect("valid setup");
    let snapshot = Snapshot::load(test_db).await.expect("snapshot");
    let search_path = snapshot.expand_search_path(&["public".to_owned()]);
    let catalog = Catalog::new(Some(Arc::new(CatalogBase::new(Arc::new(snapshot)))));
    (catalog, search_path)
}

/// The references of `sql`, each as the referencing text and what it resolves to.
fn references(catalog: &Catalog, search_path: &[String], sql: &str) -> Vec<String> {
    function_references_in(catalog, search_path, sql, None)
}

/// The references of `sql` as the body of `function`.
fn function_references_in(
    catalog: &Catalog,
    search_path: &[String],
    sql: &str,
    function: Option<&FunctionContext>,
) -> Vec<String> {
    let stmt = pgls_query::parse(sql)
        .expect("valid sql")
        .into_root()
        .expect("a single statement");
    let resolution = resolve(ResolveParams {
        stmt: &stmt,
        catalog,
        search_path,
        function,
        sql: Some(sql),
        collect_references: true,
    });
    resolution
        .references
        .iter()
        .map(|reference| {
            let text = &sql[reference.span().expect("a span")];
            assert_eq!(
                usize::from(reference.span().unwrap().start()),
                usize::try_from(reference.location()).unwrap()
            );
            let target = match reference {
                Reference::Function {
                    function,
                    arguments,
                    ..
                } => {
                    let signature = function.signature.as_ref().expect("a signature");
                    let parameters = signature
                        .arguments
                        .iter()
                        .map(|argument| {
                            argument
                                .ty
                                .clone()
                                .map(Type::Named)
                                .and_then(|ty| format_type(catalog, &ty))
                                .unwrap_or_else(|| "?".into())
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    let arguments = arguments
                        .iter()
                        .map(|argument| {
                            let name = signature.arguments[argument.parameter]
                                .name
                                .as_deref()
                                .unwrap_or("?");
                            if argument.named {
                                format!("{name} (named)")
                            } else {
                                name.to_owned()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!(
                        "function {}.{}({parameters}) <- {arguments}",
                        function.schema, function.name
                    )
                }
                Reference::Column { source, column, .. } => {
                    let source = match source {
                        ColumnSource::Relation { schema, name } => format!("{schema}.{name}"),
                        ColumnSource::Cte { name } => format!("cte {name}"),
                        ColumnSource::Subquery { alias } => {
                            format!("subquery {}", alias.as_deref().unwrap_or("?"))
                        }
                    };
                    let ty = column
                        .ty
                        .as_ref()
                        .and_then(|ty| format_type(catalog, ty))
                        .unwrap_or_else(|| "?".into());
                    let type_name = column.type_name.as_deref().unwrap_or("?");
                    format!("column {source}.{}: {ty} ({type_name})", column.name)
                }
                Reference::Relation { relation, .. } => {
                    format!("relation {}.{}", relation.schema, relation.name)
                }
                Reference::Type { type_info, .. } => {
                    format!("type {}.{}", type_info.schema, type_info.name)
                }
                Reference::Parameter { index, .. } => format!("parameter {index}"),
                Reference::EnumLiteral { type_info, .. } => {
                    format!("enum literal {}.{}", type_info.schema, type_info.name)
                }
                Reference::Star { columns, .. } => {
                    let columns = columns
                        .iter()
                        .map(|column| {
                            column
                                .qualifier
                                .iter()
                                .chain(std::iter::once(&column.name))
                                .cloned()
                                .collect::<Vec<_>>()
                                .join(".")
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("star {columns}")
                }
            };
            format!("{text} => {target}")
        })
        .collect()
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn function_references(test_db: PgPool) {
    let (catalog, search_path) = catalog(&test_db).await;
    assert_eq!(
        references(
            &catalog,
            &search_path,
            "select pick(1), public.pick('a'::text)"
        ),
        [
            "text => type pg_catalog.text",
            "pick => function public.pick(integer) <- i",
            "public.pick => function public.pick(text) <- t",
        ]
    );
    assert_eq!(
        references(
            &catalog,
            &search_path,
            "select sum_all(1, 2, 3), sum_all(1, variadic array[2, 3])"
        ),
        [
            "sum_all => function public.sum_all(integer, integer[]) <- base, nums, nums",
            "sum_all => function public.sum_all(integer, integer[]) <- base, nums",
        ]
    );
    assert_eq!(
        references(
            &catalog,
            &search_path,
            "select greet(greeting => 'hello', name => 'bob'), greet('bob')"
        ),
        [
            "greet => function public.greet(text, text) <- greeting (named), name (named)",
            "greet => function public.greet(text, text) <- name",
        ]
    );
    // An unknown literal prefers `text`.
    assert_eq!(
        references(&catalog, &search_path, "select pick('a')"),
        ["pick => function public.pick(text) <- t"]
    );
    // Ambiguous calls, and calls with arguments of unknown type, have no reference.
    assert_eq!(
        references(&catalog, &search_path, "select amb('1', '2')"),
        [""; 0]
    );
    assert_eq!(
        references(&catalog, &search_path, "select pick(x) from nope"),
        [""; 0]
    );
    assert_eq!(
        references(&catalog, &search_path, "select nope(1)"),
        [""; 0]
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn procedure_call_references(test_db: PgPool) {
    let (catalog, search_path) = catalog(&test_db).await;
    assert_eq!(
        references(&catalog, &search_path, "call do_it(pick(1), b => 'a')"),
        [
            "pick => function public.pick(integer) <- i",
            "do_it => function public.do_it(integer, text) <- a, b (named)",
        ]
    );
    // `CALL` passes an argument for output parameters too.
    assert_eq!(
        references(&catalog, &search_path, "call with_out(1, null)"),
        ["with_out => function public.with_out(integer, integer) <- a, b"]
    );
    assert_eq!(
        references(&catalog, &search_path, "call with_inout(1, null, 'x')"),
        ["with_inout => function public.with_inout(integer, integer, text) <- a, b, c"]
    );
    // Without it, the call matches no procedure.
    assert_eq!(
        references(&catalog, &search_path, "call with_out(1)"),
        [""; 0]
    );
    // Procedures created in the file.
    let mut file_catalog = catalog.clone();
    let create = pgls_query::parse(
        "create procedure file_out(x int, out y text, inout z int) language sql as 'select 1'",
    )
    .unwrap()
    .into_root()
    .unwrap();
    file_catalog.apply(&create, &search_path);
    assert_eq!(
        references(&file_catalog, &search_path, "call file_out(1, null, 2)"),
        ["file_out => function public.file_out(integer, text, integer) <- x, y, z"]
    );
    // `CALL` only calls procedures.
    assert_eq!(references(&catalog, &search_path, "call pick(1)"), [""; 0]);
}

/// A best-effort catalog keeps resolving functions and enum literals after a statement whose
/// effects are unknown, which makes the lookups of a normal catalog unknown.
#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn best_effort_catalogs_ignore_unmodelled_statements(test_db: PgPool) {
    let (catalog, search_path) = catalog(&test_db).await;
    let mut best_effort = catalog.clone().best_effort();
    let mut strict = catalog;
    let do_block = pgls_query::parse("do $$ begin end $$")
        .unwrap()
        .into_root()
        .unwrap();
    for catalog in [&mut strict, &mut best_effort] {
        catalog.apply(&do_block, &search_path);
        catalog.taint();
    }
    let sql = "select pick(1), name from users where name = 'x' and cheer('happy') = 'sad'";
    assert_eq!(
        references(&strict, &search_path, sql),
        [
            "users => relation public.users",
            "name => column public.users.name: text (text)",
            "name => column public.users.name: text (text)",
        ]
    );
    assert_eq!(
        references(&best_effort, &search_path, sql),
        [
            "users => relation public.users",
            "name => column public.users.name: text (text)",
            "name => column public.users.name: text (text)",
            "'happy' => enum literal public.mood",
            "cheer => function public.cheer(public.mood) <- m",
            "'sad' => enum literal public.mood",
            "pick => function public.pick(integer) <- i",
        ]
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn column_references(test_db: PgPool) {
    let (catalog, search_path) = catalog(&test_db).await;
    assert_eq!(
        references(
            &catalog,
            &search_path,
            "select u.name, title, public.users.id from public.users join posts p on p.user_id = users.id"
        ),
        [
            "public.users => relation public.users",
            "posts => relation public.posts",
            "p.user_id => column public.posts.user_id: bigint (int8)",
            "users.id => column public.users.id: bigint (int8)",
            "title => column public.posts.title: text (text)",
            "public.users.id => column public.users.id: bigint (int8)",
        ]
    );
    // An alias that renames a column refers to the column of the table.
    assert_eq!(
        references(
            &catalog,
            &search_path,
            "select uid from users u(uid) where u.name = ''"
        ),
        [
            "users => relation public.users",
            "uid => column public.users.id: bigint (int8)",
            "u.name => column public.users.name: text (text)",
        ]
    );
    // Ambiguous, merged, and output columns, and unknown relations have no reference.
    assert_eq!(
        references(
            &catalog,
            &search_path,
            "select id from users, posts order by id"
        ),
        [
            "users => relation public.users",
            "posts => relation public.posts"
        ]
    );
    assert_eq!(
        references(
            &catalog,
            &search_path,
            "select id, name as n from users join posts using (id) order by n"
        ),
        [
            "users => relation public.users",
            "posts => relation public.posts",
            "name => column public.users.name: text (text)",
        ]
    );
    assert_eq!(
        references(&catalog, &search_path, "select x, y.z from nope y"),
        [""; 0]
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn cte_and_subquery_columns(test_db: PgPool) {
    let (catalog, search_path) = catalog(&test_db).await;
    assert_eq!(
        references(
            &catalog,
            &search_path,
            "with c as (select id, name as n from users) select n, x.id from c x"
        ),
        [
            "users => relation public.users",
            "id => column public.users.id: bigint (int8)",
            "name => column public.users.name: text (text)",
            "n => column cte c.n: text (text)",
            "x.id => column cte c.id: bigint (int8)",
        ]
    );
    assert_eq!(
        references(
            &catalog,
            &search_path,
            "select s.total from (select count(*) as total from posts) s"
        ),
        [
            "posts => relation public.posts",
            "s.total => column subquery s.total: bigint (int8)",
        ]
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn relation_references(test_db: PgPool) {
    let (catalog, search_path) = catalog(&test_db).await;
    assert_eq!(
        references(
            &catalog,
            &search_path,
            "update posts set title = 'x' where user_id in (select id from public.users)"
        ),
        [
            "posts => relation public.posts",
            "user_id => column public.posts.user_id: bigint (int8)",
            "public.users => relation public.users",
            "id => column public.users.id: bigint (int8)",
        ]
    );
    assert_eq!(
        references(&catalog, &search_path, "delete from users u where u.id = 1"),
        [
            "users => relation public.users",
            "u.id => column public.users.id: bigint (int8)",
        ]
    );
}

/// The `*` references of `sql`.
fn stars_of(catalog: &Catalog, search_path: &[String], sql: &str) -> Vec<String> {
    references(catalog, search_path, sql)
        .into_iter()
        .filter(|reference| reference.contains("=> star "))
        .collect()
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn star_references(test_db: PgPool) {
    let (catalog, search_path) = catalog(&test_db).await;
    let stars = |sql| stars_of(&catalog, &search_path, sql);

    assert_eq!(stars("select * from users"), ["* => star id, name"]);
    // Columns are qualified when there are several FROM items.
    assert_eq!(
        stars("select *, p.* from public.users join posts p on p.user_id = users.id"),
        [
            "* => star users.id, users.name, p.id, p.user_id, p.title",
            "p.* => star p.id, p.user_id, p.title",
        ]
    );
    assert_eq!(
        stars("select public.users.* from public.users"),
        ["public.users.* => star public.users.id, public.users.name"]
    );
    // Aliases rename the columns.
    assert_eq!(stars("select * from users u(uid)"), ["* => star uid, name"]);
    // CTEs, subqueries, and RETURNING lists.
    assert_eq!(
        stars(
            "with c as (select id, name as n from users) \
             select c.*, s.* from c, (select count(*) as total from posts) s"
        ),
        ["c.* => star c.id, c.n", "s.* => star s.total",]
    );
    assert_eq!(
        stars("delete from users returning *"),
        ["* => star id, name"]
    );
    // The `*` of a subquery expands to its own FROM items.
    assert_eq!(
        stars("select exists (select * from posts where user_id = users.id) from users"),
        ["* => star id, user_id, title"]
    );
    // Joins that merge columns expand to the merged columns first, by their bare name.
    assert_eq!(
        stars("select * from users join posts using (id)"),
        ["* => star id, users.name, posts.user_id, posts.title"]
    );
    assert_eq!(
        stars("select * from users natural join people"),
        ["* => star id, users.name, people.current"]
    );
    assert_eq!(
        stars("select * from users join posts using (id) right join people using (id)"),
        ["* => star id, users.name, posts.user_id, posts.title, people.current"]
    );
    assert_eq!(
        stars("select * from (users join posts using (id)) j"),
        ["* => star id, name, user_id, title"]
    );
    // Unknown columns, duplicate column names, ambiguous merged columns, and items without a
    // name have no reference.
    assert_eq!(stars("select * from nope"), [""; 0]);
    assert_eq!(stars("select * from users, nope"), [""; 0]);
    assert_eq!(
        stars("select j.* from (users join posts on true) j"),
        [""; 0]
    );
    assert_eq!(
        stars("select * from users join posts using (id), people"),
        [""; 0]
    );
    assert_eq!(
        stars("select * from users join posts using (nope)"),
        [""; 0]
    );
    assert_eq!(stars("select * from users, (select 1 as one)"), [""; 0]);
    assert_eq!(
        stars("select 1 from users where (users.*) is not null"),
        [""; 0]
    );
}

/// The rows of `INSERT ... VALUES` are resolved more than once, and each reference is only
/// recorded once.
#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn references_are_recorded_once(test_db: PgPool) {
    let (catalog, search_path) = catalog(&test_db).await;
    assert_eq!(
        references(
            &catalog,
            &search_path,
            "insert into posts (id, user_id) values (pick(1), 2), (3, pick(4))"
        ),
        [
            "posts => relation public.posts",
            "pick => function public.pick(integer) <- i",
            "pick => function public.pick(integer) <- i",
        ]
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn type_references(test_db: PgPool) {
    let (catalog, search_path) = catalog(&test_db).await;
    assert_eq!(
        references(
            &catalog,
            &search_path,
            "select 1::int, cast(2 as pg_catalog.int8), '{}'::text[], 'sad'::mood, x::nope"
        ),
        [
            "int => type pg_catalog.int4",
            "pg_catalog.int8 => type pg_catalog.int8",
            "text => type pg_catalog.text",
            "mood => type public.mood",
            "'sad' => enum literal public.mood",
        ]
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn enum_literal_references(test_db: PgPool) {
    let (catalog, search_path) = catalog(&test_db).await;
    assert_eq!(
        references(
            &catalog,
            &search_path,
            "select id from people where current = 'happy' or 'sad' <> current \
             or current in ('happy', 'sad') or current between 'happy' and 'sad' \
             or current is distinct from 'sad' or nullif(current, 'sad') is null"
        ),
        [
            "people => relation public.people",
            "id => column public.people.id: bigint (int8)",
            "current => column public.people.current: public.mood (mood)",
            "current => column public.people.current: public.mood (mood)",
            "current => column public.people.current: public.mood (mood)",
            "current => column public.people.current: public.mood (mood)",
            "current => column public.people.current: public.mood (mood)",
            "current => column public.people.current: public.mood (mood)",
            "'happy' => enum literal public.mood",
            "'sad' => enum literal public.mood",
            "'happy' => enum literal public.mood",
            "'sad' => enum literal public.mood",
            "'happy' => enum literal public.mood",
            "'sad' => enum literal public.mood",
            "'sad' => enum literal public.mood",
            "'sad' => enum literal public.mood",
        ]
    );
    assert_eq!(
        references(
            &catalog,
            &search_path,
            "insert into people (current) values ('happy'), ('sad')"
        ),
        [
            "people => relation public.people",
            "'happy' => enum literal public.mood",
            "'sad' => enum literal public.mood",
        ]
    );
    assert_eq!(
        references(
            &catalog,
            &search_path,
            "update people set current = 'sad' where id = 1"
        ),
        [
            "people => relation public.people",
            "'sad' => enum literal public.mood",
            "id => column public.people.id: bigint (int8)",
        ]
    );
    assert_eq!(
        references(
            &catalog,
            &search_path,
            "select cheer('happy'), cheer_all('happy', 'sad')"
        ),
        [
            "'happy' => enum literal public.mood",
            "cheer => function public.cheer(public.mood) <- m",
            "'happy' => enum literal public.mood",
            "'sad' => enum literal public.mood",
            "cheer_all => function public.cheer_all(public.mood[]) <- moods, moods",
        ]
    );
    // Literals that are not coerced to an enum are no enum literals.
    assert_eq!(
        references(
            &catalog,
            &search_path,
            "select 'happy', current::text = 'happy' from people"
        ),
        [
            "people => relation public.people",
            "current => column public.people.current: public.mood (mood)",
            "text => type pg_catalog.text",
        ]
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn parameter_references(test_db: PgPool) {
    let (catalog, search_path) = catalog(&test_db).await;
    let param = |name: &str| FunctionParam {
        name: Some(name.to_owned()),
        type_schema: None,
        type_name: "int8".to_owned(),
        is_array: false,
    };
    let function = FunctionContext {
        function_name: "lookup".to_owned(),
        params: vec![param("user_id"), param("name")],
    };
    assert_eq!(
        function_references_in(
            &catalog,
            &search_path,
            "select title from posts where user_id = $1 or id = lookup.user_id or id = name",
            Some(&function),
        ),
        [
            "posts => relation public.posts",
            "title => column public.posts.title: text (text)",
            "user_id => column public.posts.user_id: bigint (int8)",
            "$1 => parameter 0",
            "id => column public.posts.id: bigint (int8)",
            "lookup.user_id => parameter 0",
            "id => column public.posts.id: bigint (int8)",
            "name => parameter 1",
        ]
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn references_are_only_collected_on_request(test_db: PgPool) {
    let (catalog, search_path) = catalog(&test_db).await;
    let sql = "select pick(1), name from users";
    let stmt = pgls_query::parse(sql).unwrap().into_root().unwrap();
    let params = |collect_references| ResolveParams {
        stmt: &stmt,
        catalog: &catalog,
        search_path: &search_path,
        function: None,
        sql: Some(sql),
        collect_references,
    };
    let without = resolve(params(false));
    let with = resolve(params(true));
    assert!(without.references.is_empty());
    assert_eq!(with.references.len(), 3);
    assert_eq!(with.findings, without.findings);
    assert_eq!(with.database_only, without.database_only);

    // `CALL` only gives references, never findings.
    let sql = "call do_it(nope(1), 1 + 'x')";
    let stmt = pgls_query::parse(sql).unwrap().into_root().unwrap();
    let params = |collect_references| ResolveParams {
        stmt: &stmt,
        catalog: &catalog,
        search_path: &search_path,
        function: None,
        sql: Some(sql),
        collect_references,
    };
    let without = resolve(params(false));
    let with = resolve(params(true));
    assert!(with.findings.is_empty());
    assert_eq!(with.findings, without.findings);
    assert_eq!(with.database_only, without.database_only);
}
