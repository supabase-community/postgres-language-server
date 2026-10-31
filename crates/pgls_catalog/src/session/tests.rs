use pgls_query::{NodeEnum, protobuf::RangeVar};

use super::{Session, TransactionMode};
use crate::Catalog;

/// Applies a statement with a catalog that knows nothing but the statement, so unqualified
/// names are in the first schema of the search path.
fn apply(s: &mut Session, q: &str) {
    let n = pgls_query::parse(q).unwrap().into_root().unwrap();
    let mut catalog = Catalog::new(None);
    catalog.apply(&n, s.search_path());
    s.apply(&n, &catalog);
}
fn dangerous(s: &Session, q: &str) -> bool {
    let n = pgls_query::parse(q).unwrap().into_root().unwrap();
    s.is_dangerous_lock_stmt(&n, &Catalog::new(None))
}
fn range_var(schema: &str, name: &str) -> RangeVar {
    RangeVar {
        schemaname: schema.into(),
        relname: name.into(),
        relpersistence: "p".into(),
        ..Default::default()
    }
}
fn created(s: &Session, schema: &str, name: &str) -> bool {
    s.has_created_relation(&range_var(schema, name), &Catalog::new(None))
}
fn not_valid(s: &Session, table: &str, name: &str) -> bool {
    s.has_not_valid_constraint(&range_var("", table), name, &Catalog::new(None))
}

#[test]
fn transaction_boundaries_reset_state_and_timeouts() {
    let mut s = Session::new(vec!["public".into()]);
    apply(&mut s, "SET lock_timeout='1s'");
    apply(&mut s, "CREATE TABLE made (id int)");
    assert!(s.has_lock_timeout() && created(&s, "", "made"));
    apply(&mut s, "BEGIN");
    assert_eq!(s.transaction_depth(), 1);
    apply(&mut s, "COMMIT");
    assert_eq!(s.transaction_depth(), 0);
    assert!(!s.has_lock_timeout() && !created(&s, "", "made"));
    apply(&mut s, "SET statement_timeout='1s'");
    apply(&mut s, "ROLLBACK");
    assert!(!s.has_statement_timeout());
    apply(&mut s, "SET lock_timeout='1s'");
    apply(&mut s, "SET statement_timeout='1s'");
    apply(&mut s, "SET idle_in_transaction_session_timeout='1s'");
    assert!(
        s.has_lock_timeout() && s.has_statement_timeout() && s.has_idle_in_transaction_timeout()
    );
    apply(&mut s, "RESET ALL");
    assert!(
        !s.has_lock_timeout() && !s.has_statement_timeout() && !s.has_idle_in_transaction_timeout()
    );
}
#[test]
fn created_objects_and_schema_use_search_path() {
    let mut empty_path = Session::default();
    apply(&mut empty_path, "CREATE TABLE public_default (id int)");
    assert!(created(&empty_path, "", "public_default"));
    let mut s = Session::new(vec!["app".into(), "public".into()]);
    apply(&mut s, "CREATE TABLE made (id int)");
    assert!(created(&s, "", "made"));
    assert!(!created(&s, "public", "made"));
    apply(&mut s, "CREATE INDEX made_idx ON made (id)");
    assert!(s.locks.has_created_object("app", "made_idx"));
}
#[test]
fn names_are_compared_exactly() {
    // The parser folds unquoted names to lower case, so `"Users"` and `users` are different
    // tables.
    let mut s = Session::new(vec!["public".into()]);
    apply(&mut s, r#"CREATE TABLE "Users" (id int)"#);
    apply(&mut s, r#"CREATE TABLE "App".events (id int)"#);
    apply(
        &mut s,
        r#"ALTER TABLE "Things" ADD CONSTRAINT "Things_fk" FOREIGN KEY (id) REFERENCES other(id) NOT VALID"#,
    );
    assert!(created(&s, "", "Users"));
    assert!(!created(&s, "", "users"));
    assert!(created(&s, "App", "events"));
    assert!(!created(&s, "app", "events"));
    assert!(not_valid(&s, "Things", "Things_fk"));
    assert!(!not_valid(&s, "things", "Things_fk"));
    assert!(!not_valid(&s, "Things", "things_fk"));
}
#[test]
fn not_valid_and_access_exclusive_are_tracked() {
    let mut s = Session::new(vec!["app".into()]);
    apply(
        &mut s,
        "ALTER TABLE things ADD CONSTRAINT things_fk FOREIGN KEY (id) REFERENCES other(id) NOT VALID",
    );
    assert!(not_valid(&s, "things", "things_fk"));
    let parsed = pgls_query::parse("ALTER TABLE things ADD COLUMN name text")
        .unwrap()
        .into_root()
        .unwrap();
    let NodeEnum::AlterTableStmt(stmt) = parsed else {
        panic!()
    };
    assert_eq!(
        s.access_exclusive_table_for_alter(&stmt, &Catalog::new(None)),
        Some(("app".into(), "things".into()))
    );
    s.apply(&NodeEnum::AlterTableStmt(stmt), &Catalog::new(None));
    assert!(s.is_holding_access_exclusive());
    assert_eq!(s.access_exclusive_tables().len(), 1);
    apply(&mut s, "CREATE TABLE fresh (id int)");
    let fresh = pgls_query::parse("ALTER TABLE fresh ADD COLUMN x int")
        .unwrap()
        .into_root()
        .unwrap();
    let NodeEnum::AlterTableStmt(stmt) = fresh else {
        panic!()
    };
    assert_eq!(
        s.access_exclusive_table_for_alter(&stmt, &Catalog::new(None)),
        None
    );
    let validate = pgls_query::parse("ALTER TABLE things VALIDATE CONSTRAINT x")
        .unwrap()
        .into_root()
        .unwrap();
    let NodeEnum::AlterTableStmt(stmt) = validate else {
        panic!()
    };
    assert_eq!(
        s.access_exclusive_table_for_alter(&stmt, &Catalog::new(None)),
        None
    );
}
#[test]
fn dangerous_lock_statements() {
    let s = Session::new(vec!["public".into()]);
    for q in [
        "ALTER TABLE t ADD COLUMN x int",
        "CREATE INDEX i ON t(id)",
        "DROP TABLE t",
        "TRUNCATE t",
        "VACUUM (FULL) t",
        "REINDEX TABLE t",
        "ALTER TABLE t RENAME TO x",
        "REFRESH MATERIALIZED VIEW t",
    ] {
        assert!(dangerous(&s, q), "{q}");
    }
    for q in [
        "CREATE INDEX CONCURRENTLY i ON t(id)",
        "VACUUM t",
        "REINDEX TABLE CONCURRENTLY t",
        "REFRESH MATERIALIZED VIEW CONCURRENTLY t",
    ] {
        assert!(!dangerous(&s, q), "{q}");
    }
}
#[test]
fn search_path_and_role_settings() {
    let mut s = Session::new(vec!["initial".into()]);
    apply(&mut s, "SET search_path TO app, public");
    assert_eq!(s.search_path(), ["app", "public"]);
    apply(&mut s, "SET search_path = DEFAULT");
    assert_eq!(s.search_path(), ["initial"]);
    apply(
        &mut s,
        "SET search_path TO \"QuotedSchema\", 'other schema'",
    );
    assert_eq!(s.search_path(), ["QuotedSchema", "other schema"]);
    apply(&mut s, "BEGIN");
    apply(&mut s, "SET search_path TO changed");
    apply(&mut s, "ROLLBACK");
    assert_eq!(s.search_path(), ["QuotedSchema", "other schema"]);
    apply(&mut s, "BEGIN");
    apply(&mut s, "SET LOCAL search_path TO local_schema");
    apply(&mut s, "COMMIT");
    assert_eq!(s.search_path(), ["QuotedSchema", "other schema"]);
    apply(&mut s, "BEGIN");
    apply(&mut s, "SET LOCAL search_path TO local_schema");
    apply(&mut s, "ROLLBACK");
    assert_eq!(s.search_path(), ["QuotedSchema", "other schema"]);
    apply(&mut s, "RESET search_path");
    assert_eq!(s.search_path(), ["initial"]);
    apply(&mut s, "SET search_path TO another");
    apply(&mut s, "RESET ALL");
    assert_eq!(s.search_path(), ["initial"]);
    apply(&mut s, "SET ROLE some_role");
    assert_eq!(s.role(), Some("some_role"));
    apply(&mut s, "SET ROLE NONE");
    assert_eq!(s.role(), None);
    apply(&mut s, "SET ROLE some_role");
    apply(&mut s, "BEGIN");
    apply(&mut s, "SET ROLE another_role");
    apply(&mut s, "ROLLBACK");
    assert_eq!(s.role(), Some("some_role"));
    apply(&mut s, "SET ROLE some_role");
    apply(&mut s, "RESET ROLE");
    assert_eq!(s.role(), None);
}

#[test]
fn rollback_forgets_local_role() {
    let mut s = Session::new(vec!["public".into()]);
    apply(&mut s, "SET ROLE outer_role");
    apply(&mut s, "BEGIN");
    apply(&mut s, "SET LOCAL ROLE local_role");
    apply(&mut s, "ROLLBACK");
    assert_eq!(s.role(), Some("outer_role"));
    apply(&mut s, "BEGIN");
    apply(&mut s, "SET ROLE committed_role");
    apply(&mut s, "COMMIT");
    assert_eq!(s.role(), Some("committed_role"));
}

#[test]
fn check_function_bodies() {
    let mut s = Session::new(vec!["public".into()]);
    assert!(s.check_function_bodies());
    apply(&mut s, "SET check_function_bodies = false");
    assert!(!s.check_function_bodies());
    apply(&mut s, "SET check_function_bodies TO on");
    assert!(s.check_function_bodies());
    apply(&mut s, "SET check_function_bodies = 0");
    assert!(!s.check_function_bodies());
    apply(&mut s, "RESET check_function_bodies");
    assert!(s.check_function_bodies());
    apply(&mut s, "BEGIN");
    apply(&mut s, "SET LOCAL check_function_bodies = off");
    assert!(!s.check_function_bodies());
    apply(&mut s, "COMMIT");
    assert!(s.check_function_bodies());
    apply(&mut s, "BEGIN");
    apply(&mut s, "SET check_function_bodies = off");
    apply(&mut s, "ROLLBACK");
    assert!(s.check_function_bodies());
    apply(&mut s, "SET check_function_bodies = off");
    apply(&mut s, "RESET ALL");
    assert!(s.check_function_bodies());
}

#[test]
fn autocommit_releases_locks_after_each_statement() {
    let mut s =
        Session::new(vec!["public".into()]).with_transaction_mode(TransactionMode::Autocommit);
    assert_eq!(s.in_transaction(), Some(false));
    apply(&mut s, "SET lock_timeout='1s'");
    apply(&mut s, "SET LOCAL search_path = other");
    apply(&mut s, "CREATE TABLE made (id int)");
    apply(
        &mut s,
        "ALTER TABLE users ADD CONSTRAINT c CHECK (id > 0) NOT VALID",
    );
    // Session-level settings persist, the locks and SET LOCAL end with the statement.
    assert!(s.has_lock_timeout() && created(&s, "", "made"));
    assert_eq!(s.search_path(), ["public"]);
    assert!(!s.is_holding_access_exclusive() && s.access_exclusive_tables().is_empty());
    assert!(!not_valid(&s, "users", "c"));

    apply(&mut s, "BEGIN");
    assert_eq!(s.in_transaction(), Some(true));
    apply(&mut s, "ALTER TABLE users ADD COLUMN name text");
    assert!(s.is_holding_access_exclusive());
    apply(&mut s, "COMMIT");
    assert_eq!(s.in_transaction(), Some(false));
    assert!(!s.is_holding_access_exclusive());
}

#[test]
fn transaction_mode_decides_whether_statements_run_in_a_transaction() {
    let mut s = Session::new(vec![]);
    assert_eq!(s.in_transaction(), None);
    apply(&mut s, "BEGIN");
    assert_eq!(s.in_transaction(), Some(true));

    let mut s = Session::new(vec![]).with_transaction_mode(TransactionMode::Transaction);
    assert_eq!(s.in_transaction(), Some(true));
    apply(&mut s, "ALTER TABLE users ADD COLUMN name text");
    assert!(s.is_holding_access_exclusive());
}

#[test]
fn implicit_block_runs_statements_in_a_transaction_until_the_query_ends() {
    let mut s =
        Session::new(vec!["public".into()]).with_transaction_mode(TransactionMode::Autocommit);
    s.begin_implicit_block();
    assert_eq!(s.in_transaction(), Some(true));
    apply(&mut s, "ALTER TABLE users ADD COLUMN name text");
    apply(&mut s, "SET LOCAL search_path = other");
    assert!(s.is_holding_access_exclusive());
    assert_eq!(s.search_path(), ["other"]);

    // A COMMIT in the query ends the block, but the next statement starts a new one.
    apply(&mut s, "COMMIT");
    assert_eq!(s.in_transaction(), Some(true));
    apply(&mut s, "ALTER TABLE users ADD COLUMN email text");
    assert!(s.is_holding_access_exclusive());

    s.end_implicit_block();
    assert_eq!(s.in_transaction(), Some(false));
    assert!(!s.is_holding_access_exclusive());
    assert_eq!(s.search_path(), ["public"]);

    // An explicit transaction started in the query stays open after it.
    s.begin_implicit_block();
    apply(&mut s, "BEGIN");
    apply(&mut s, "ALTER TABLE users ADD COLUMN age int");
    s.end_implicit_block();
    assert_eq!(s.in_transaction(), Some(true));
    assert!(s.is_holding_access_exclusive());
}
#[test]
fn created_relations_are_resolved_with_the_catalog() {
    use std::sync::Arc;

    // `app` is on the search path, but doesn't exist.
    let snapshot = crate::Snapshot {
        schemas: ["public", "pg_catalog"]
            .map(|name| crate::Schema {
                name: name.into(),
                ..Default::default()
            })
            .into(),
        tables: vec![crate::Table {
            id: 1,
            schema: "public".into(),
            name: "users".into(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let mut catalog = Catalog::new(Some(Arc::new(crate::CatalogBase::new(Arc::new(snapshot)))));
    let mut s = Session::new(vec!["app".into(), "public".into()]);
    let run = |s: &mut Session, catalog: &mut Catalog, q: &str| {
        let n = pgls_query::parse(q).unwrap().into_root().unwrap();
        let dangerous = s.is_dangerous_lock_stmt(&n, catalog);
        catalog.apply(&n, s.search_path());
        s.apply(&n, catalog);
        dangerous
    };

    run(&mut s, &mut catalog, "BEGIN");
    // Postgres creates `made` in `public`, the first schema of the search path that exists.
    run(&mut s, &mut catalog, "CREATE TABLE made (id int)");
    assert!(s.has_created_relation(&range_var("public", "made"), &catalog));
    assert!(s.has_created_relation(&range_var("", "made"), &catalog));
    run(
        &mut s,
        &mut catalog,
        "CREATE TABLE public.qualified (id int)",
    );
    assert!(!run(
        &mut s,
        &mut catalog,
        "ALTER TABLE qualified ADD COLUMN name text"
    ));
    assert!(run(
        &mut s,
        &mut catalog,
        "ALTER TABLE users ADD COLUMN email text"
    ));
    assert_eq!(
        s.access_exclusive_tables(),
        [("public".to_string(), "users".to_string())]
    );
}
