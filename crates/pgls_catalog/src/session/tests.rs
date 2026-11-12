use pgls_query::{NodeEnum, protobuf::RangeVar};

use super::{LockMode, Session, TransactionMode, alter_table_lock};
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
fn altered(s: &Session, schema: &str, name: &str) -> bool {
    s.has_altered_table(&range_var(schema, name), &Catalog::new(None))
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
    assert!(s.in_explicit_transaction());
    apply(&mut s, "COMMIT");
    assert!(!s.in_explicit_transaction());
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
fn altered_tables_use_the_search_path_of_their_statement() {
    let mut s = Session::new(vec!["public".into()]);
    apply(&mut s, "ALTER TABLE things ADD COLUMN a int");
    apply(&mut s, "SET search_path = app");
    assert!(!altered(&s, "", "things"));
    assert!(altered(&s, "public", "things"));
    apply(&mut s, "BEGIN");
    apply(&mut s, "ALTER TABLE things ADD COLUMN b int");
    apply(&mut s, "COMMIT");
    assert!(altered(&s, "app", "things"));
}
#[test]
fn not_valid_and_access_exclusive_are_tracked() {
    let mut s = Session::new(vec!["app".into()]);
    apply(
        &mut s,
        "ALTER TABLE things ADD CONSTRAINT things_fk FOREIGN KEY (id) REFERENCES other(id) NOT VALID",
    );
    assert!(not_valid(&s, "things", "things_fk"));
    // A foreign key takes SHARE ROW EXCLUSIVE on both tables.
    assert!(!s.is_holding_access_exclusive());
    let locks = |s: &Session, q: &str| {
        let n = pgls_query::parse(q).unwrap().into_root().unwrap();
        s.table_locks(&n, &Catalog::new(None))
            .into_iter()
            .map(|lock| (lock.mode, lock.schema, lock.name))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        locks(&s, "ALTER TABLE things ADD COLUMN name text"),
        [(LockMode::AccessExclusive, "app".into(), "things".into())]
    );
    apply(&mut s, "ALTER TABLE things ADD COLUMN name text");
    assert!(s.is_holding_access_exclusive());
    assert_eq!(s.access_exclusive_tables().len(), 1);
    apply(&mut s, "CREATE TABLE fresh (id int)");
    assert_eq!(locks(&s, "ALTER TABLE fresh ADD COLUMN x int"), []);
    assert_eq!(
        locks(&s, "ALTER TABLE things VALIDATE CONSTRAINT x"),
        [(
            LockMode::ShareUpdateExclusive,
            "app".into(),
            "things".into()
        )]
    );
    // Attaching a partition locks the partition ACCESS EXCLUSIVE.
    assert_eq!(
        locks(
            &s,
            "ALTER TABLE things ATTACH PARTITION things_2024 FOR VALUES FROM (1) TO (2)"
        ),
        [
            (
                LockMode::ShareUpdateExclusive,
                "app".into(),
                "things".into()
            ),
            (
                LockMode::AccessExclusive,
                "app".into(),
                "things_2024".into()
            )
        ]
    );
    assert_eq!(
        locks(
            &s,
            "ALTER TABLE things ADD CONSTRAINT f FOREIGN KEY (id) REFERENCES public.other (id)"
        ),
        [
            (LockMode::ShareRowExclusive, "app".into(), "things".into()),
            (LockMode::ShareRowExclusive, "public".into(), "other".into())
        ]
    );
    assert_eq!(
        locks(&s, "CREATE INDEX CONCURRENTLY i ON things (id)"),
        [(
            LockMode::ShareUpdateExclusive,
            "app".into(),
            "things".into()
        )]
    );
}

#[test]
fn alter_table_takes_the_strongest_lock_of_its_subcommands() {
    let lock = |q: &str| {
        let Some(NodeEnum::AlterTableStmt(stmt)) = pgls_query::parse(q).unwrap().into_root() else {
            panic!("not an ALTER TABLE: {q}")
        };
        alter_table_lock(&stmt)
    };
    use LockMode::*;
    for (q, mode) in [
        ("ALTER TABLE t ADD COLUMN a int", AccessExclusive),
        ("ALTER TABLE t VALIDATE CONSTRAINT c", ShareUpdateExclusive),
        (
            "ALTER TABLE t ALTER COLUMN a SET STATISTICS 100",
            ShareUpdateExclusive,
        ),
        (
            "ALTER TABLE t ALTER COLUMN a SET (n_distinct = 10)",
            ShareUpdateExclusive,
        ),
        ("ALTER TABLE t CLUSTER ON i", ShareUpdateExclusive),
        ("ALTER TABLE t SET WITHOUT CLUSTER", ShareUpdateExclusive),
        ("ALTER TABLE t ENABLE TRIGGER tr", ShareRowExclusive),
        ("ALTER TABLE t ENABLE ALWAYS TRIGGER tr", ShareRowExclusive),
        ("ALTER TABLE t ENABLE REPLICA TRIGGER tr", ShareRowExclusive),
        ("ALTER TABLE t DISABLE TRIGGER ALL", ShareRowExclusive),
        (
            "ALTER TABLE t ADD CONSTRAINT f FOREIGN KEY (a) REFERENCES o (id)",
            ShareRowExclusive,
        ),
        (
            "ALTER TABLE t ADD CONSTRAINT c CHECK (a > 0)",
            AccessExclusive,
        ),
        ("ALTER TABLE t ADD CONSTRAINT u UNIQUE (a)", AccessExclusive),
        (
            "ALTER TABLE t ATTACH PARTITION p FOR VALUES IN (1)",
            ShareUpdateExclusive,
        ),
        ("ALTER TABLE t DETACH PARTITION p", AccessExclusive),
        (
            "ALTER TABLE t DETACH PARTITION p CONCURRENTLY",
            ShareUpdateExclusive,
        ),
        (
            "ALTER TABLE t DETACH PARTITION p FINALIZE",
            ShareUpdateExclusive,
        ),
        ("ALTER TABLE t SET (fillfactor = 70)", ShareUpdateExclusive),
        (
            "ALTER TABLE t RESET (autovacuum_enabled)",
            ShareUpdateExclusive,
        ),
        (
            "ALTER TABLE t SET (toast.autovacuum_enabled = false)",
            ShareUpdateExclusive,
        ),
        (
            "ALTER TABLE t SET (user_catalog_table = true)",
            AccessExclusive,
        ),
        (
            "ALTER VIEW v SET (security_barrier = true)",
            AccessExclusive,
        ),
        (
            "ALTER TABLE t SET (fillfactor = 70, user_catalog_table = true)",
            AccessExclusive,
        ),
        // Postgres rejects the option later, without a stronger lock.
        (
            "ALTER TABLE t SET (no_such_option = 1)",
            ShareUpdateExclusive,
        ),
        ("ALTER TABLE t ALTER COLUMN a SET NOT NULL", AccessExclusive),
        ("ALTER TABLE t OWNER TO r", AccessExclusive),
        // The strongest lock of the subcommands.
        (
            "ALTER TABLE t VALIDATE CONSTRAINT c, ENABLE TRIGGER tr",
            ShareRowExclusive,
        ),
        (
            "ALTER TABLE t VALIDATE CONSTRAINT c, ADD COLUMN a int",
            AccessExclusive,
        ),
    ] {
        assert_eq!(lock(q), mode, "{q}");
    }
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
        "ALTER TABLE t ENABLE TRIGGER tr",
        // The partition is locked ACCESS EXCLUSIVE.
        "ALTER TABLE t ATTACH PARTITION p FOR VALUES IN (1)",
    ] {
        assert!(dangerous(&s, q), "{q}");
    }
    for q in [
        "CREATE INDEX CONCURRENTLY i ON t(id)",
        "VACUUM t",
        "REINDEX TABLE CONCURRENTLY t",
        "REFRESH MATERIALIZED VIEW CONCURRENTLY t",
        // SHARE UPDATE EXCLUSIVE doesn't block reads or writes.
        "ALTER TABLE t VALIDATE CONSTRAINT c",
        "ALTER TABLE t SET (autovacuum_enabled = false)",
        "ALTER TABLE t DETACH PARTITION p CONCURRENTLY",
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
fn transaction_blocks_follow_postgres() {
    let mut s =
        Session::new(vec!["public".into()]).with_transaction_mode(TransactionMode::Autocommit);
    let opens = |s: &Session, q: &str| {
        s.opens_transaction_block(&pgls_query::parse(q).unwrap().into_root().unwrap())
    };
    // A `BEGIN` in a transaction block only warns, and one `COMMIT` ends the block.
    assert!(opens(&s, "BEGIN"));
    apply(&mut s, "BEGIN");
    assert!(!opens(&s, "BEGIN"));
    apply(&mut s, "BEGIN");
    apply(&mut s, "COMMIT");
    assert_eq!(s.in_transaction(), Some(false));

    // Savepoints don't open or end the block.
    apply(&mut s, "SAVEPOINT a");
    assert_eq!(s.in_transaction(), Some(false));
    apply(&mut s, "BEGIN");
    apply(&mut s, "SAVEPOINT a");
    apply(&mut s, "RELEASE SAVEPOINT a");
    assert_eq!(s.in_transaction(), Some(true));
    apply(&mut s, "SAVEPOINT b");
    apply(&mut s, "COMMIT");
    assert_eq!(s.in_transaction(), Some(false));

    // `AND CHAIN` ends the transaction and opens the next block right away.
    apply(&mut s, "BEGIN");
    apply(&mut s, "ALTER TABLE users ADD COLUMN name text");
    apply(&mut s, "SET LOCAL search_path = other");
    assert!(opens(&s, "COMMIT AND CHAIN") && opens(&s, "ROLLBACK AND CHAIN"));
    assert!(!opens(&s, "COMMIT"));
    apply(&mut s, "COMMIT AND CHAIN");
    assert_eq!(s.in_transaction(), Some(true));
    assert!(!s.is_holding_access_exclusive());
    assert_eq!(s.search_path(), ["public"]);
    apply(&mut s, "SET search_path = other");
    apply(&mut s, "ROLLBACK AND CHAIN");
    assert_eq!(s.in_transaction(), Some(true));
    assert_eq!(s.search_path(), ["public"]);

    // `PREPARE TRANSACTION` ends the block and keeps its settings, like `COMMIT`.
    apply(&mut s, "SET search_path = other");
    apply(&mut s, "PREPARE TRANSACTION 'tx'");
    assert_eq!(s.in_transaction(), Some(false));
    assert_eq!(s.search_path(), ["other"]);

    // Outside of a transaction block, Postgres rejects `AND CHAIN`.
    assert!(!opens(&s, "COMMIT AND CHAIN"));
    apply(&mut s, "COMMIT AND CHAIN");
    assert_eq!(s.in_transaction(), Some(false));
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
/// A catalog over a snapshot with the schemas, `pg_catalog`, and the table `public.users`.
fn catalog_with_users(schemas: &[&str]) -> Catalog {
    use std::sync::Arc;

    let snapshot = crate::Snapshot {
        schemas: schemas
            .iter()
            .chain(&["pg_catalog"])
            .map(|name| crate::Schema {
                name: (*name).into(),
                ..Default::default()
            })
            .collect(),
        tables: vec![crate::Table {
            id: 1,
            schema: "public".into(),
            name: "users".into(),
            ..Default::default()
        }],
        ..Default::default()
    };
    Catalog::new(Some(Arc::new(crate::CatalogBase::new(Arc::new(snapshot)))))
}
/// Applies a statement to the catalog and the session, like the analyser. Returns whether the
/// statement was a dangerous lock statement.
fn run(s: &mut Session, catalog: &mut Catalog, q: &str) -> bool {
    let n = pgls_query::parse(q).unwrap().into_root().unwrap();
    let dangerous = s.is_dangerous_lock_stmt(&n, catalog);
    catalog.apply(&n, s.search_path());
    s.apply(&n, catalog);
    dangerous
}
#[test]
fn created_relations_are_resolved_with_the_catalog() {
    // `app` is on the search path, but doesn't exist.
    let mut catalog = catalog_with_users(&["public"]);
    let mut s = Session::new(vec!["app".into(), "public".into()]);

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
#[test]
fn skipped_if_not_exists_creates_nothing() {
    // Postgres skips both statements: `users` exists in `public`, and indexes share their names
    // with tables.
    let mut catalog = catalog_with_users(&["public"]);
    let mut s = Session::new(vec!["app".into(), "public".into()]);
    run(&mut s, &mut catalog, "BEGIN");
    run(
        &mut s,
        &mut catalog,
        "CREATE TABLE IF NOT EXISTS users (id bigint)",
    );
    run(
        &mut s,
        &mut catalog,
        "CREATE INDEX IF NOT EXISTS users ON public.users (id)",
    );
    run(
        &mut s,
        &mut catalog,
        "CREATE TABLE IF NOT EXISTS copy AS SELECT 1 AS id",
    );
    run(
        &mut s,
        &mut catalog,
        "CREATE TABLE IF NOT EXISTS copy AS SELECT 1 AS id",
    );
    assert!(!s.has_created_relation(&range_var("public", "users"), &catalog));
    assert!(run(
        &mut s,
        &mut catalog,
        "ALTER TABLE public.users ADD COLUMN payload text"
    ));
    assert!(s.has_created_relation(&range_var("public", "copy"), &catalog));
}
#[test]
fn temporary_tables_dont_capture_new_tables() {
    // Without `TEMP`, Postgres creates the table in the first schema of the search path, even
    // if a temporary table of that name exists.
    let mut catalog = Catalog::new(None);
    let mut s = Session::new(vec!["public".into()]);
    run(&mut s, &mut catalog, "BEGIN");
    run(&mut s, &mut catalog, "CREATE TEMP TABLE events (id bigint)");
    run(&mut s, &mut catalog, "CREATE TABLE events (id bigint)");
    run(&mut s, &mut catalog, "CREATE TEMP TABLE copy (id bigint)");
    run(&mut s, &mut catalog, "CREATE TABLE copy AS SELECT 1 AS id");
    assert!(!run(
        &mut s,
        &mut catalog,
        "ALTER TABLE public.events ADD COLUMN payload text"
    ));
    assert!(!run(
        &mut s,
        &mut catalog,
        "ALTER TABLE public.copy ADD COLUMN payload text"
    ));
    assert!(!s.is_holding_access_exclusive());
}
#[test]
fn index_names_are_resolved_with_the_catalog() {
    // Postgres creates the index in `public`, the schema of its table, and `ALTER INDEX` finds
    // it there through the search path.
    let mut catalog = catalog_with_users(&["app", "public"]);
    let mut s = Session::new(vec!["app".into(), "public".into()]);
    run(&mut s, &mut catalog, "BEGIN");
    run(
        &mut s,
        &mut catalog,
        "CREATE INDEX users_id_idx ON users (id)",
    );
    assert!(!run(
        &mut s,
        &mut catalog,
        "ALTER INDEX users_id_idx SET (fillfactor = 70)"
    ));
    assert!(s.has_created_relation(&range_var("", "users_id_idx"), &catalog));
    assert!(!s.is_holding_access_exclusive());
}
