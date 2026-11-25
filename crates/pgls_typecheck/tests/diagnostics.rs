#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "a test fails on unexpected data; clippy exempts `#[test]` functions, not their helpers"
)]

use pgls_console::fmt::{Formatter, HTML};
use pgls_diagnostics::Diagnostic;
use pgls_typecheck::{TypecheckParams, check_sql};
use sqlx::{Executor, PgPool, postgres::PgPoolOptions};
use std::fmt::Write;

struct TestSetup<'a> {
    name: &'a str,
    query: &'a str,
    setup: Option<&'a str>,
    test_db: &'a PgPool,
}

impl TestSetup<'_> {
    /// A pool of one connection without a statement cache, like the language server's statement
    /// pool.
    async fn pool(test_db: &PgPool) -> PgPool {
        PgPoolOptions::new()
            .max_connections(1)
            .connect_with(
                (*test_db.connect_options())
                    .clone()
                    .statement_cache_capacity(0),
            )
            .await
            .expect("Failed to connect")
    }

    async fn test(self) {
        if let Some(setup) = self.setup {
            self.test_db
                .execute(setup)
                .await
                .expect("Failed to setup test selfbase");
        }

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .expect("Error loading sql language");

        let root = pgls_query::parse(self.query)
            .unwrap()
            .into_root()
            .expect("Failed to parse query");
        let tree = parser.parse(self.query, None).unwrap();

        let pool = Self::pool(self.test_db).await;
        let result = check_sql(TypecheckParams {
            conn: &pool,
            sql: self.query,
            ast: &root,
            tree: &tree,
            search_path: &[],
            duplicate_assignments_reported: false,
        })
        .await;
        pool.close().await;

        assert!(
            result.is_ok(),
            "Got Typechecking error: {}",
            result.unwrap_err()
        );

        let maybe_diagnostic = result.unwrap();

        let content = match maybe_diagnostic {
            Some(d) => {
                let mut result = String::new();

                if let Some(span) = d.location().span {
                    for (idx, c) in self.query.char_indices() {
                        if pgls_text_size::TextSize::new(idx.try_into().unwrap()) == span.start() {
                            result.push_str("~~~");
                        }
                        if pgls_text_size::TextSize::new(idx.try_into().unwrap()) == span.end() {
                            result.push_str("~~~");
                        }
                        result.push(c);
                    }
                } else {
                    result.push_str("~~~");
                    result.push_str(self.query);
                    result.push_str("~~~");
                }

                writeln!(&mut result).unwrap();
                writeln!(&mut result).unwrap();

                let mut msg_content = vec![];
                let mut writer = HTML::new(&mut msg_content);
                let mut formatter = Formatter::new(&mut writer);
                d.message(&mut formatter).unwrap();

                result.push_str(String::from_utf8(msg_content).unwrap().as_str());

                result
            }
            None => String::from("No Diagnostic"),
        };

        insta::with_settings!({
            prepend_module_to_snapshot => false,
        }, {
            insta::assert_snapshot!(self.name, content);

        });
    }
}

#[pgls_test_utils::db_test]
async fn invalid_column(test_db: PgPool) {
    TestSetup {
        name: "invalid_column",
        query: "select id, unknown from contacts;",
        setup: Some(
            r#"
        create table public.contacts (
            id serial primary key,
            name varchar(255) not null,
            is_vegetarian bool default false,
            middle_name varchar(255)
        );
    "#,
        ),
        test_db: &test_db,
    }
    .test()
    .await;
}

#[pgls_test_utils::db_test]
async fn create_table_as_invalid_column(test_db: PgPool) {
    TestSetup {
        name: "create_table_as_invalid_column",
        query: "CREATE TABLE t2 AS SELECT t.nope FROM t;",
        setup: Some("CREATE TABLE t (a INT);"),
        test_db: &test_db,
    }
    .test()
    .await;
}

#[pgls_test_utils::db_test]
async fn create_table_as_invalid_relation(test_db: PgPool) {
    TestSetup {
        name: "create_table_as_invalid_relation",
        query: "CREATE TABLE t3 AS SELECT t.a FROM nonexistent_table AS t;",
        setup: Some("CREATE TABLE t (a INT);"),
        test_db: &test_db,
    }
    .test()
    .await;
}

#[pgls_test_utils::db_test]
async fn create_materialized_view_as_invalid_column(test_db: PgPool) {
    TestSetup {
        name: "create_materialized_view_as_invalid_column",
        query: "CREATE MATERIALIZED VIEW mv1 AS SELECT t.nope FROM t;",
        setup: Some("CREATE TABLE t (a INT);"),
        test_db: &test_db,
    }
    .test()
    .await;
}

#[pgls_test_utils::db_test]
async fn create_view_as_invalid_column(test_db: PgPool) {
    TestSetup {
        name: "create_view_as_invalid_column",
        query: "CREATE VIEW v1 AS SELECT t.nope FROM t;",
        setup: Some("CREATE TABLE t (a INT);"),
        test_db: &test_db,
    }
    .test()
    .await;
}

#[pgls_test_utils::db_test]
async fn create_as_valid_queries_do_not_execute_ddl(test_db: PgPool) {
    test_db
        .execute("CREATE TABLE t (a INT);")
        .await
        .expect("Failed to create test table");

    for (name, query) in [
        (
            "create_table_as_valid",
            "CREATE TABLE t2 AS SELECT t.a FROM t;",
        ),
        (
            "create_materialized_view_as_valid",
            "CREATE MATERIALIZED VIEW mv1 AS SELECT t.a FROM t;",
        ),
        (
            "create_view_as_valid",
            "CREATE VIEW v1 AS SELECT t.a FROM t;",
        ),
    ] {
        TestSetup {
            name,
            query,
            setup: None,
            test_db: &test_db,
        }
        .test()
        .await;
    }

    for object_name in ["t2", "mv1", "v1"] {
        let object = sqlx::query_scalar::<_, Option<String>>("SELECT to_regclass($1)::text")
            .bind(object_name)
            .fetch_one(&test_db)
            .await
            .expect("Failed to check created object");
        assert_eq!(object, None, "{object_name} should not have been created");
    }
}

#[pgls_test_utils::db_test]
async fn merge_invalid_column(test_db: PgPool) {
    TestSetup {
        name: "merge_invalid_column",
        query: "merge into users u using imports i on i.id = u.id when matched then update set name = i.nope;",
        setup: Some(
            r#"
        create table public.users (id int8 primary key, name text);
        create table public.imports (id int8, name text);
    "#,
        ),
        test_db: &test_db,
    }
    .test()
    .await;
}

#[pgls_test_utils::db_test]
async fn merge_is_not_executed(test_db: PgPool) {
    test_db
        .execute(
            "create table public.users (id int8 primary key, name text);
             create table public.imports (id int8, name text);
             insert into public.imports values (1, 'a');",
        )
        .await
        .expect("Failed to set up the tables");

    TestSetup {
        name: "merge_valid",
        query: "merge into users u using imports i on i.id = u.id \
                when matched then update set name = i.name \
                when not matched then insert values (i.id, i.name);",
        setup: None,
        test_db: &test_db,
    }
    .test()
    .await;

    let rows = sqlx::query_scalar::<_, i64>("select count(*) from public.users")
        .fetch_one(&test_db)
        .await
        .expect("Failed to count the rows");
    assert_eq!(rows, 0, "the MERGE must not insert the row");
}

/// Postgres reports a column assigned twice without a position. Once
/// `banDuplicateColumnAssignments` reported it, only the other errors of the statement remain.
#[pgls_test_utils::db_test]
async fn duplicate_assignments_are_reported_once(test_db: PgPool) {
    test_db
        .execute(
            "create table public.users (id int8 primary key, name text);
             create table public.imports (id int8, name text);",
        )
        .await
        .expect("Failed to set up the tables");

    for sql in [
        "merge into users u using imports i on u.id = i.id \
         when matched then update set name = i.name, name = 'again'",
        "update users set name = 'a', name = 'b'",
    ] {
        let reported = check(&test_db, sql, false).await;
        assert!(
            reported
                .as_deref()
                .is_some_and(|message| message.contains("multiple assignments to same column")),
            "{sql}: {reported:?}"
        );
        assert_eq!(check(&test_db, sql, true).await, None, "{sql}");
    }
    // Other errors of the statement are still reported.
    let other = check(
        &test_db,
        "merge into users u using imports i on u.id = i.id \
         when matched then update set name = i.nope, name = 'again'",
        true,
    )
    .await;
    assert!(
        other
            .as_deref()
            .is_some_and(|message| message.contains("i.nope")),
        "{other:?}"
    );
}

/// The message of the error of `sql`, if any.
async fn check(
    test_db: &PgPool,
    sql: &str,
    duplicate_assignments_reported: bool,
) -> Option<String> {
    let root = pgls_query::parse(sql).unwrap().into_root().unwrap();
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
        .unwrap();
    let tree = parser.parse(sql, None).unwrap();
    let pool = TestSetup::pool(test_db).await;
    let message = check_sql(TypecheckParams {
        conn: &pool,
        sql,
        ast: &root,
        tree: &tree,
        search_path: &[],
        duplicate_assignments_reported,
    })
    .await
    .unwrap()
    .map(|diagnostic| pgls_diagnostics::PrintDescription(&diagnostic).to_string());
    pool.close().await;
    message
}

/// A check leaves neither its prepared statement nor its search path in the session of the
/// connection, which goes back to the pool.
#[pgls_test_utils::db_test]
async fn the_session_is_reset_after_a_check(test_db: PgPool) {
    test_db
        .execute("create schema other; create table other.items (id int);")
        .await
        .expect("Failed to set up the table");
    // One connection, so that the queries below run in the session of the checks.
    let pool = TestSetup::pool(&test_db).await;
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
        .unwrap();

    for (sql, reported) in [
        ("select id from items", false),
        ("select nope from items", true),
    ] {
        let root = pgls_query::parse(sql).unwrap().into_root().unwrap();
        let tree = parser.parse(sql, None).unwrap();
        let diagnostic = check_sql(TypecheckParams {
            conn: &pool,
            sql,
            ast: &root,
            tree: &tree,
            search_path: &["other".to_string()],
            duplicate_assignments_reported: false,
        })
        .await
        .unwrap();
        assert_eq!(diagnostic.is_some(), reported, "{sql}: {diagnostic:?}");
    }

    let prepared: i64 = sqlx::query_scalar("select count(*) from pg_prepared_statements")
        .persistent(false)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(prepared, 0);
    let search_path: String = sqlx::query_scalar("show search_path")
        .persistent(false)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(search_path, r#""$user", public"#);
    pool.close().await;
}

/// A check that is dropped half-way, e.g. because another check of the same pull failed, closes
/// its connection instead of returning it to the pool with its search path set.
#[pgls_test_utils::db_test]
async fn a_dropped_check_does_not_return_its_session(test_db: PgPool) {
    test_db
        .execute("create schema other; create table other.items (id int);")
        .await
        .expect("Failed to set up the table");
    let pool = TestSetup::pool(&test_db).await;
    let sql = "select id from items";
    let root = pgls_query::parse(sql).unwrap().into_root().unwrap();
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
        .unwrap();
    let tree = parser.parse(sql, None).unwrap();

    // The lock makes the check wait in `prepare`, after it set the search path.
    let mut lock = test_db.begin().await.unwrap();
    lock.execute("lock table other.items in access exclusive mode")
        .await
        .unwrap();
    let search_path = ["other".to_string()];
    let mut check = Box::pin(check_sql(TypecheckParams {
        conn: &pool,
        sql,
        ast: &root,
        tree: &tree,
        search_path: &search_path,
        duplicate_assignments_reported: false,
    }));
    // 100 polls of 50 ms: the check waits on the lock within a few round trips.
    let polls_max = 100;
    let mut waiting = false;
    for _ in 0..polls_max {
        tokio::select! {
            result = &mut check => panic!("the check didn't wait for the lock: {result:?}"),
            () = tokio::time::sleep(std::time::Duration::from_millis(50)) => {}
        }
        let blocked: i64 = sqlx::query_scalar("select count(*) from pg_locks where not granted")
            .fetch_one(&test_db)
            .await
            .unwrap();
        if blocked > 0 {
            waiting = true;
            break;
        }
    }
    assert!(waiting, "the check never waited for the lock");
    drop(check);
    lock.rollback().await.unwrap();

    let search_path: String = sqlx::query_scalar("show search_path")
        .persistent(false)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(search_path, r#""$user", public"#);
    pool.close().await;
}
