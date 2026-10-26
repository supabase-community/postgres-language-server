use std::sync::Arc;

use sqlx::{Executor, PgPool};

use super::{ColumnSource, Reference, ResolveParams, resolve};
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
    let stmt = pgls_query::parse(sql)
        .expect("valid sql")
        .into_root()
        .expect("a single statement");
    let resolution = resolve(ResolveParams {
        stmt: &stmt,
        catalog,
        search_path,
        function: None,
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
}
