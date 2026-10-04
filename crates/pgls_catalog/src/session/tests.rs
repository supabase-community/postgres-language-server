use pgls_query::NodeEnum;

use super::Session;

fn apply(s: &mut Session, q: &str) {
    let n = pgls_query::parse(q).unwrap().into_root().unwrap();
    s.apply(&n);
}
fn dangerous(s: &Session, q: &str) -> bool {
    let n = pgls_query::parse(q).unwrap().into_root().unwrap();
    s.is_dangerous_lock_stmt(&n)
}

#[test]
fn transaction_boundaries_reset_state_and_timeouts() {
    let mut s = Session::new(vec!["public".into()]);
    apply(&mut s, "SET lock_timeout='1s'");
    apply(&mut s, "CREATE TABLE made (id int)");
    assert!(s.has_lock_timeout() && s.has_created_object("", "made"));
    apply(&mut s, "BEGIN");
    assert_eq!(s.transaction_depth(), 1);
    apply(&mut s, "COMMIT");
    assert_eq!(s.transaction_depth(), 0);
    assert!(!s.has_lock_timeout() && !s.has_created_object("", "made"));
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
    assert!(empty_path.has_created_object("", "public_default"));
    let mut s = Session::new(vec!["app".into(), "public".into()]);
    apply(&mut s, "CREATE TABLE made (id int)");
    assert!(s.has_created_object("", "made"));
    assert!(!s.has_created_object("public", "made"));
    apply(&mut s, "CREATE INDEX made_idx ON made (id)");
    assert!(s.has_created_object("app", "made_idx"));
}
#[test]
fn not_valid_and_access_exclusive_are_tracked() {
    let mut s = Session::new(vec!["app".into()]);
    apply(
        &mut s,
        "ALTER TABLE things ADD CONSTRAINT things_fk FOREIGN KEY (id) REFERENCES other(id) NOT VALID",
    );
    assert!(s.has_not_valid_constraint("", "things", "things_fk"));
    let parsed = pgls_query::parse("ALTER TABLE things ADD COLUMN name text")
        .unwrap()
        .into_root()
        .unwrap();
    let NodeEnum::AlterTableStmt(stmt) = parsed else {
        panic!()
    };
    assert_eq!(
        s.access_exclusive_table_for_alter(&stmt),
        Some(("app".into(), "things".into()))
    );
    s.apply(&NodeEnum::AlterTableStmt(stmt));
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
    assert_eq!(s.access_exclusive_table_for_alter(&stmt), None);
    let validate = pgls_query::parse("ALTER TABLE things VALIDATE CONSTRAINT x")
        .unwrap()
        .into_root()
        .unwrap();
    let NodeEnum::AlterTableStmt(stmt) = validate else {
        panic!()
    };
    assert_eq!(s.access_exclusive_table_for_alter(&stmt), None);
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
