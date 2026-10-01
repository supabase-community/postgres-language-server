use std::sync::Arc;

use crate::Snapshot;

use super::{Catalog, CatalogBase};
use crate::lookup::{CatalogView, FunctionKind, Lookup, Origin, RelationKind};

fn schema(name: &str) -> crate::Schema {
    crate::Schema {
        name: name.into(),
        ..Default::default()
    }
}

/// A database with the schemas `public` and `app`, the table `public.users (id, name)`, and the
/// function `public.greet(text, text default)`.
fn base() -> Arc<CatalogBase> {
    let schemas = vec![schema("public"), schema("app"), schema("pg_catalog")];
    let tables = vec![crate::Table {
        id: 1,
        schema: "public".into(),
        name: "users".into(),
        ..Default::default()
    }];
    let columns = ["id", "name"]
        .into_iter()
        .enumerate()
        .map(|(i, name)| crate::Column {
            name: name.into(),
            table_name: "users".into(),
            table_oid: 1,
            class_kind: crate::ColumnClassKind::OrdinaryTable,
            number: i as i64 + 1,
            schema_name: "public".into(),
            type_id: 25,
            type_name: Some("text".into()),
            is_nullable: true,
            is_primary_key: false,
            is_unique: false,
            default_expr: None,
            varchar_length: None,
            comment: None,
        })
        .collect();
    let functions = vec![crate::Function {
        schema: "public".into(),
        name: "greet".into(),
        args: crate::FunctionArgs {
            args: vec![
                crate::FunctionArg {
                    mode: "in".into(),
                    name: "first".into(),
                    type_id: 25,
                    has_default: Some(false),
                },
                crate::FunctionArg {
                    mode: "in".into(),
                    name: "last".into(),
                    type_id: 25,
                    has_default: Some(true),
                },
            ],
        },
        ..Default::default()
    }];
    let cache = Snapshot {
        schemas,
        tables,
        columns,
        functions,
        typing_metadata: true,
        ..Default::default()
    };
    Arc::new(CatalogBase::new(Arc::new(cache)))
}

fn path(schemas: &[&str]) -> Vec<String> {
    schemas.iter().map(|s| s.to_string()).collect()
}

fn catalog_after(sql: &str, search_path: &[String]) -> Catalog {
    let mut catalog = Catalog::new(Some(base()));
    let parsed = pgls_query::parse(sql).expect("valid sql");
    for stmt in parsed.stmts() {
        catalog.apply(stmt, search_path);
    }
    catalog
}

fn column_names(
    catalog: &Catalog,
    schema: Option<&str>,
    name: &str,
    search_path: &[String],
) -> Option<Vec<String>> {
    catalog
        .relation(schema, name, search_path)
        .found()
        .and_then(|relation| relation.columns)
        .map(|columns| columns.into_iter().map(|c| c.name).collect())
}

#[test]
fn database_objects_are_found() {
    let search_path = path(&["public"]);
    let catalog = Catalog::new(Some(base()));

    let users = catalog
        .relation(None, "users", &search_path)
        .found()
        .unwrap();
    assert_eq!(users.origin, Origin::Database);
    assert_eq!(
        column_names(&catalog, None, "users", &search_path).unwrap(),
        ["id", "name"]
    );
    assert_eq!(
        catalog.relation(None, "nope", &search_path),
        Lookup::Missing
    );
    assert_eq!(
        catalog.relation(Some("app"), "users", &search_path),
        Lookup::Missing
    );

    let Lookup::Found(greet) = catalog.functions(None, "greet", &search_path) else {
        panic!("greet is missing");
    };
    assert_eq!((greet[0].min_args, greet[0].max_args), (1, Some(2)));
    assert_eq!(catalog.schema("app"), Lookup::Found(()));
    assert_eq!(catalog.schema("nope"), Lookup::Missing);
}

#[test]
fn without_database_everything_is_unknown() {
    let catalog = Catalog::new(None);
    assert_eq!(
        catalog.relation(Some("public"), "users", &[]),
        Lookup::Unknown
    );
    assert_eq!(catalog.functions(None, "greet", &[]), Lookup::Unknown);
    assert_eq!(catalog.schema("public"), Lookup::Unknown);
}

#[test]
fn temporary_tables_live_in_pg_temp() {
    let search_path = path(&["public"]);
    let catalog = catalog_after("create temp table scratch (id int);", &search_path);
    let scratch = catalog
        .relation(None, "scratch", &search_path)
        .found()
        .unwrap();
    assert_eq!(scratch.schema, "pg_temp");
    assert_eq!(scratch.origin, Origin::File);
    assert!(
        catalog
            .relation(Some("pg_temp"), "scratch", &search_path)
            .found()
            .is_some()
    );
    assert_eq!(
        catalog.relation(Some("public"), "scratch", &search_path),
        Lookup::Missing
    );
}

#[test]
fn unqualified_objects_are_created_in_the_first_existing_schema() {
    let search_path = path(&["$user", "missing", "app", "public"]);
    let catalog = catalog_after("create table item (id int);", &search_path);
    assert!(
        catalog
            .relation(Some("app"), "item", &search_path)
            .found()
            .is_some()
    );
}

#[test]
fn dml_between_ddl_changes_nothing() {
    let search_path = path(&["public"]);
    let catalog = catalog_after(
        "insert into users values (1, 'a'); update users set name = 'b'; delete from users; select 1; set lock_timeout = '1s';",
        &search_path,
    );
    assert!(!catalog.is_tainted());
    assert_eq!(
        catalog.relation(None, "nope", &search_path),
        Lookup::Missing
    );
}

#[test]
fn table_columns_follow_like_inherits_and_alter_table() {
    let search_path = path(&["public"]);
    let catalog = catalog_after(
        "create table base_t (a int, b int);
         create table child (c int) inherits (base_t);
         create table copy (like users, extra int);
         alter table base_t add column d int, drop column b;
         alter table child rename column c to e;
         alter table users alter column name type varchar(10);",
        &search_path,
    );
    assert_eq!(
        column_names(&catalog, None, "base_t", &search_path).unwrap(),
        ["a", "d"]
    );
    assert_eq!(
        column_names(&catalog, None, "child", &search_path).unwrap(),
        ["a", "e", "d"]
    );
    assert_eq!(
        column_names(&catalog, None, "copy", &search_path).unwrap(),
        ["id", "name", "extra"]
    );
    let users = catalog
        .relation(None, "users", &search_path)
        .found()
        .unwrap();
    assert_eq!(users.origin, Origin::File);
    assert_eq!(
        users.columns.unwrap()[1].type_name.as_deref(),
        Some("varchar")
    );
}

#[test]
fn partitions_and_typed_tables_inherit_columns() {
    let search_path = path(&["public"]);
    let catalog = catalog_after(
        "create table events (id int, at timestamptz) partition by range (at);
         create table events_2024 partition of events for values from ('2024-01-01') to ('2025-01-01');
         create type pair as (l int, r int);
         create table pairs of pair;",
        &search_path,
    );
    let events = catalog
        .relation(None, "events", &search_path)
        .found()
        .unwrap();
    assert_eq!(events.kind, RelationKind::PartitionedTable);
    assert_eq!(
        column_names(&catalog, None, "events_2024", &search_path).unwrap(),
        ["id", "at"]
    );
    assert_eq!(
        column_names(&catalog, None, "pairs", &search_path).unwrap(),
        ["l", "r"]
    );
}

#[test]
fn derived_columns_of_views_and_table_as() {
    let search_path = path(&["public"]);
    let catalog = catalog_after(
        "create view v1 as select * from users;
         create view v2 (ident) as select u.id, upper(name), 1, name::text as n2, case when true then 1 end, now()::date from users u;
         create table t3 as select users.* from users;
         create materialized view m4 as select id from users union select 2;
         select id, name into t5 from users;
         create view v6 as select * from users join users u2 using (id);
         create view v7 as values (1, 2);
         create view v8 as select exists (select 1), array[1], coalesce(1, 2), greatest(1), current_date;",
        &search_path,
    );
    assert_eq!(
        column_names(&catalog, None, "v1", &search_path).unwrap(),
        ["id", "name"]
    );
    assert_eq!(
        column_names(&catalog, None, "v2", &search_path).unwrap(),
        ["ident", "upper", "?column?", "n2", "case", "now"]
    );
    assert_eq!(
        column_names(&catalog, None, "t3", &search_path).unwrap(),
        ["id", "name"]
    );
    let m4 = catalog.relation(None, "m4", &search_path).found().unwrap();
    assert_eq!(m4.kind, RelationKind::MaterializedView);
    assert_eq!(m4.columns.unwrap()[0].name, "id");
    assert_eq!(
        column_names(&catalog, None, "t5", &search_path).unwrap(),
        ["id", "name"]
    );
    // `USING (id)` merges `id`, but `name` appears twice, which Postgres rejects.
    assert!(
        catalog
            .relation(None, "v6", &search_path)
            .found()
            .unwrap()
            .columns
            .is_none()
    );
    assert_eq!(
        column_names(&catalog, None, "v7", &search_path).unwrap(),
        ["column1", "column2"]
    );
    assert_eq!(
        column_names(&catalog, None, "v8", &search_path).unwrap(),
        ["exists", "array", "coalesce", "greatest", "current_date"]
    );
}

#[test]
fn renames_and_schema_moves() {
    let search_path = path(&["public"]);
    let catalog = catalog_after(
        "alter table users rename to people;
         create type mood as enum ('ok');
         alter type mood rename to feeling;
         alter type feeling set schema app;
         create function f() returns int language sql as 'select 1';
         alter function f() rename to g;
         alter function g set schema app;
         create table moved (id int);
         alter table moved set schema app;",
        &search_path,
    );
    assert_eq!(
        catalog.relation(None, "users", &search_path),
        Lookup::Missing
    );
    assert!(
        catalog
            .relation(None, "people", &search_path)
            .found()
            .is_some()
    );
    assert_eq!(catalog.type_(None, "mood", &search_path), Lookup::Missing);
    assert_eq!(
        catalog.type_(None, "feeling", &search_path),
        Lookup::Missing
    );
    assert!(
        catalog
            .type_(Some("app"), "feeling", &search_path)
            .found()
            .is_some()
    );
    assert_eq!(catalog.functions(None, "f", &search_path), Lookup::Missing);
    assert_eq!(catalog.functions(None, "g", &search_path), Lookup::Missing);
    assert!(matches!(
        catalog.functions(Some("app"), "g", &search_path),
        Lookup::Found(_)
    ));
    assert_eq!(
        catalog.relation(None, "moved", &search_path),
        Lookup::Missing
    );
    assert!(
        catalog
            .relation(Some("app"), "moved", &search_path)
            .found()
            .is_some()
    );
}

#[test]
fn drops() {
    let search_path = path(&["public"]);
    let catalog = catalog_after(
        "drop table users;
         create type shape as (x int);
         drop type shape;
         drop function greet;
         drop schema app cascade;",
        &search_path,
    );
    assert_eq!(
        catalog.relation(None, "users", &search_path),
        Lookup::Missing
    );
    assert_eq!(catalog.type_(None, "users", &search_path), Lookup::Missing);
    assert_eq!(catalog.type_(None, "shape", &search_path), Lookup::Missing);
    assert_eq!(
        catalog.functions(None, "greet", &search_path),
        Lookup::Missing
    );
    assert_eq!(catalog.schema("app"), Lookup::Missing);
}

#[test]
fn dropped_schema_hides_database_objects_after_recreation() {
    let search_path = path(&["public"]);
    let catalog = catalog_after(
        "drop schema public cascade; create schema public;",
        &search_path,
    );
    assert_eq!(catalog.schema("public"), Lookup::Found(()));
    assert_eq!(
        catalog.relation(None, "users", &search_path),
        Lookup::Missing
    );
}

#[test]
fn function_overloads() {
    let search_path = path(&["public"]);
    let catalog = catalog_after(
        "create function greet(a int, b int, c int) returns int language sql as 'select 1';
         create function sum_all(variadic xs int[]) returns int language sql as 'select 1';
         create function pairs(out l int, out r int) returns setof record language sql as 'select 1, 2';
         create function users_of() returns setof users language sql as 'select * from users';
         create procedure p(a int) language sql as 'select 1';
         create aggregate my_agg(int) (sfunc = int4pl, stype = int);",
        &search_path,
    );
    let Lookup::Found(greet) = catalog.functions(None, "greet", &search_path) else {
        panic!("greet is missing");
    };
    assert_eq!(greet.len(), 2);

    let Lookup::Found(sum_all) = catalog.functions(None, "sum_all", &search_path) else {
        panic!("sum_all is missing");
    };
    assert_eq!((sum_all[0].min_args, sum_all[0].max_args), (1, None));

    let Lookup::Found(pairs) = catalog.functions(None, "pairs", &search_path) else {
        panic!("pairs is missing");
    };
    assert!(pairs[0].returns_set);
    let columns: Vec<_> = pairs[0]
        .return_columns
        .iter()
        .flatten()
        .map(|c| c.name.as_str())
        .collect();
    assert_eq!(columns, ["l", "r"]);

    let Lookup::Found(users_of) = catalog.functions(None, "users_of", &search_path) else {
        panic!("users_of is missing");
    };
    assert_eq!(users_of[0].return_columns.as_ref().unwrap().len(), 2);

    let Lookup::Found(p) = catalog.functions(None, "p", &search_path) else {
        panic!("p is missing");
    };
    assert_eq!(p[0].kind, FunctionKind::Procedure);
    assert!(matches!(
        catalog.functions(None, "my_agg", &search_path),
        Lookup::Found(_)
    ));
}

#[test]
fn dropping_one_of_several_overloads() {
    let search_path = path(&["public"]);
    let catalog = catalog_after(
        "create function greet(a int, b int, c int) returns int language sql as 'select 1';
         drop function greet(int, int, int);",
        &search_path,
    );
    let Lookup::Found(greet) = catalog.functions(None, "greet", &search_path) else {
        panic!("greet is missing");
    };
    assert_eq!(greet.len(), 1);
    assert_eq!(greet[0].origin, Origin::Database);

    // With two overloads of the same arity, we can't tell which one was dropped.
    let catalog = catalog_after(
        "create function twice(a int) returns int language sql as 'select 1';
         create function twice(a text) returns int language sql as 'select 1';
         drop function twice(text);",
        &search_path,
    );
    assert_eq!(
        catalog.functions(None, "twice", &search_path),
        Lookup::Unknown
    );
}

#[test]
fn range_types_define_constructors() {
    let search_path = path(&["public"]);
    let catalog = catalog_after(
        "create type floatrange as range (subtype = float8);",
        &search_path,
    );
    assert!(
        catalog
            .type_(None, "floatrange", &search_path)
            .found()
            .is_some()
    );
    assert!(
        catalog
            .type_(None, "floatmultirange", &search_path)
            .found()
            .is_some()
    );
    let Lookup::Found(constructors) = catalog.functions(None, "floatrange", &search_path) else {
        panic!("constructor is missing");
    };
    assert_eq!(
        (constructors[0].min_args, constructors[0].max_args),
        (2, Some(3))
    );
}

#[test]
fn create_schema_elements_go_into_the_schema() {
    let search_path = path(&["public"]);
    let catalog = catalog_after(
        "create schema reporting create table daily (id int) create view recent as select * from daily;",
        &search_path,
    );
    assert_eq!(catalog.schema("reporting"), Lookup::Found(()));
    assert_eq!(
        column_names(&catalog, Some("reporting"), "recent", &search_path).unwrap(),
        ["id"]
    );
}

#[test]
fn unmodelled_statements_taint_the_catalog() {
    let search_path = path(&["public"]);
    for sql in [
        "do $$ begin create table x (); end $$;",
        "call something();",
        "create extension not_installed;",
        "alter schema app rename to other;",
    ] {
        let catalog = catalog_after(sql, &search_path);
        assert!(catalog.is_tainted(), "{sql}");
        assert_eq!(
            catalog.relation(None, "x", &search_path),
            Lookup::Unknown,
            "{sql}"
        );
        // Known objects are still found.
        assert!(
            catalog
                .relation(None, "users", &search_path)
                .found()
                .is_some()
        );
    }
}

#[test]
fn composite_types_are_not_missing_relations() {
    let search_path = path(&["public"]);
    let catalog = catalog_after("create type point2 as (x int, y int);", &search_path);
    assert_eq!(
        catalog.relation(None, "point2", &search_path),
        Lookup::Unknown
    );
    let point = catalog.type_(None, "point2", &search_path).found().unwrap();
    assert_eq!(point.attributes.unwrap().len(), 2);
}

#[test]
fn if_not_exists_keeps_existing_relations() {
    let search_path = path(&["public"]);
    let catalog = catalog_after(
        "create table if not exists users (other int);",
        &search_path,
    );
    assert_eq!(
        column_names(&catalog, None, "users", &search_path).unwrap(),
        ["id", "name"]
    );
    assert_eq!(
        catalog
            .relation(None, "users", &search_path)
            .found()
            .unwrap()
            .origin,
        Origin::Database
    );
}

#[test]
fn rollback_restores_the_catalog() {
    let search_path = path(&["public"]);
    let catalog = catalog_after(
        "begin; alter table users rename to people; create table scratch (id int); rollback;",
        &search_path,
    );
    assert!(
        catalog
            .relation(None, "users", &search_path)
            .found()
            .is_some()
    );
    assert_eq!(
        catalog.relation(None, "people", &search_path),
        Lookup::Missing
    );
    assert_eq!(
        catalog.relation(None, "scratch", &search_path),
        Lookup::Missing
    );

    let catalog = catalog_after(
        "begin; create table kept (id int); savepoint s; drop table users; rollback to savepoint s; commit;",
        &search_path,
    );
    assert!(
        catalog
            .relation(None, "users", &search_path)
            .found()
            .is_some()
    );
    assert!(
        catalog
            .relation(None, "kept", &search_path)
            .found()
            .is_some()
    );
}

#[test]
fn column_changes_make_database_children_unknown() {
    let search_path = path(&["public"]);
    let tables = ["events", "events_2024"]
        .into_iter()
        .enumerate()
        .map(|(i, name)| crate::Table {
            id: i as i64 + 1,
            schema: "public".into(),
            name: name.into(),
            is_inheritance_child: name == "events_2024",
            ..Default::default()
        })
        .collect();
    let cache = Snapshot {
        schemas: vec![schema("public")],
        tables,
        ..Default::default()
    };
    let mut catalog = Catalog::new(Some(Arc::new(CatalogBase::new(Arc::new(cache)))));
    let parsed = pgls_query::parse("alter table events add column at timestamptz").unwrap();
    catalog.apply(parsed.stmts()[0], &search_path);
    let child = catalog
        .relation(None, "events_2024", &search_path)
        .found()
        .unwrap();
    assert!(child.columns.is_none());
}

fn table_columns(snapshot: &Snapshot, schema: &str, table: &str) -> Vec<String> {
    let mut columns: Vec<_> = snapshot
        .columns
        .iter()
        .filter(|column| column.schema_name == schema && column.table_name == table)
        .collect();
    columns.sort_by_key(|column| column.number);
    columns.iter().map(|column| column.name.clone()).collect()
}

#[test]
fn snapshot_without_changes_is_the_database_snapshot() {
    let base = base();
    let catalog = Catalog::new(Some(Arc::clone(&base)));
    assert!(Arc::ptr_eq(&catalog.snapshot().unwrap(), base.snapshot()));
    assert!(Catalog::new(None).snapshot().is_none());
}

#[test]
fn snapshot_includes_the_changes_of_the_file() {
    let search_path = path(&["public"]);
    let catalog = catalog_after(
        "create schema reporting;
         create table reporting.daily (day date, total int8);
         alter table users add column email text;
         alter table users drop column name;
         create type mood as enum ('happy', 'sad');
         create function shout(text) returns text language sql as 'select $1';
         create sequence counter;",
        &search_path,
    );
    let snapshot = catalog.snapshot().unwrap();

    assert!(
        snapshot
            .schemas
            .iter()
            .any(|schema| schema.name == "reporting")
    );
    let daily = snapshot
        .tables
        .iter()
        .find(|table| table.name == "daily")
        .unwrap();
    assert_eq!(daily.schema, "reporting");
    assert!(daily.id < 0);
    assert_eq!(
        table_columns(&snapshot, "reporting", "daily"),
        ["day", "total"]
    );

    // Columns that stay keep their database definition.
    assert_eq!(table_columns(&snapshot, "public", "users"), ["id", "email"]);
    let id = snapshot
        .columns
        .iter()
        .find(|column| column.table_name == "users" && column.name == "id")
        .unwrap();
    assert_eq!(id.table_oid, 1);
    assert_eq!(id.type_id, 25);

    assert!(snapshot.types.iter().any(|type_| type_.name == "mood"));
    assert!(
        snapshot
            .functions
            .iter()
            .any(|function| function.name == "shout")
    );
    assert!(
        snapshot
            .functions
            .iter()
            .any(|function| function.name == "greet")
    );
    assert!(
        snapshot
            .sequences
            .iter()
            .any(|sequence| sequence.name == "counter")
    );
}

#[test]
fn snapshot_excludes_dropped_objects() {
    let search_path = path(&["public"]);
    let catalog = catalog_after(
        "drop table users; drop function greet; drop schema app;",
        &search_path,
    );
    let snapshot = catalog.snapshot().unwrap();
    assert!(snapshot.tables.is_empty());
    assert!(snapshot.columns.is_empty());
    assert!(snapshot.functions.is_empty());
    assert!(!snapshot.schemas.iter().any(|schema| schema.name == "app"));

    let catalog = catalog_after("alter table users rename to people;", &search_path);
    let snapshot = catalog.snapshot().unwrap();
    let tables: Vec<_> = snapshot.tables.iter().map(|table| &table.name).collect();
    assert_eq!(tables, ["people"]);
    assert_eq!(table_columns(&snapshot, "public", "people"), ["id", "name"]);
}
#[cfg(feature = "db")]
mod typing_overlay_tests {
    use super::path;
    use crate::lookup::{CatalogView, Lookup};
    use crate::typing::{Type, TypeId, TypeKind};
    use crate::{Catalog, CatalogBase, Snapshot};
    use sqlx::PgPool;
    use std::sync::Arc;

    async fn database_catalog(test_db: &PgPool) -> Catalog {
        let snapshot = Snapshot::load(test_db).await.expect("load snapshot");
        Catalog::new(Some(Arc::new(CatalogBase::new(Arc::new(snapshot)))))
    }

    fn apply(catalog: &mut Catalog, sql: &str, search_path: &[String]) {
        let parsed = pgls_query::parse(sql).expect("valid sql");
        for stmt in parsed.stmts() {
            catalog.apply(stmt, search_path);
        }
    }

    fn type_id(catalog: &Catalog, name: &str, path: &[String]) -> TypeId {
        catalog
            .type_(Some("pg_catalog"), name, path)
            .found()
            .and_then(|ty| ty.id)
            .unwrap_or_else(|| panic!("missing pg_catalog.{name}"))
    }

    fn column_type(catalog: &Catalog, table: &str, column: &str, path: &[String]) -> Option<Type> {
        catalog
            .relation(None, table, path)
            .found()?
            .columns?
            .into_iter()
            .find(|item| item.name == column)?
            .ty
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn created_table_columns_have_type_identities(test_db: PgPool) {
        let path = path(&["public"]);
        let mut catalog = database_catalog(&test_db).await;
        apply(
            &mut catalog,
            "create type mood as enum ('ok');
             create domain positive_int as int check (value > 0);
             create type pair as (left_value int, right_value text);
             create table typed (
               i int, b bigint, t text, v varchar(10), n numeric(5,2), z timestamptz,
               a int[], aa int[][], s serial, bs bigserial, ss smallserial,
               m mood, d positive_int, p pair
             );
             create table typed_copy (like typed);
             create table more (value int);
             alter table more alter column value type bigint;
             alter table more rename column value to renamed;",
            &path,
        );
        let expected = [
            ("i", "int4"),
            ("b", "int8"),
            ("t", "text"),
            ("v", "varchar"),
            ("n", "numeric"),
            ("z", "timestamptz"),
            ("a", "_int4"),
            ("aa", "_int4"),
            ("s", "int4"),
            ("bs", "int8"),
            ("ss", "int2"),
        ];
        for (column, pg_type) in expected {
            assert_eq!(
                column_type(&catalog, "typed", column, &path),
                Some(Type::Named(type_id(&catalog, pg_type, &path))),
                "typed.{column}"
            );
        }
        for (column, type_name) in [("m", "mood"), ("d", "positive_int"), ("p", "pair")] {
            let expected = catalog
                .type_(None, type_name, &path)
                .found()
                .unwrap()
                .id
                .unwrap();
            assert_eq!(
                column_type(&catalog, "typed", column, &path),
                Some(Type::Named(expected))
            );
        }
        for (column, pg_type) in expected {
            assert_eq!(
                column_type(&catalog, "typed_copy", column, &path),
                Some(Type::Named(type_id(&catalog, pg_type, &path))),
                "LIKE copied {column}"
            );
        }
        assert_eq!(
            column_type(&catalog, "more", "renamed", &path),
            Some(Type::Named(type_id(&catalog, "int8", &path)))
        );
        let mood = catalog.type_(None, "mood", &path).found().unwrap();
        let mood_id = mood.id.clone().unwrap();
        assert_eq!(mood.kind, Some(TypeKind::Enum));
        assert_eq!(mood.category, Some('E'));
        let mood_array = catalog.type_(None, "_mood", &path).found().unwrap();
        assert_eq!(mood_array.element, Some(mood_id.clone()));
        assert_eq!(mood.array, mood_array.id);
        let domain = catalog.type_(None, "positive_int", &path).found().unwrap();
        assert_eq!(domain.kind, Some(TypeKind::Domain));
        assert_eq!(domain.base, Some(type_id(&catalog, "int4", &path)));
        let pair = catalog.type_(None, "pair", &path).found().unwrap();
        assert_eq!(pair.kind, Some(TypeKind::Composite));
        assert_eq!(pair.category, Some('C'));
        assert_eq!(pair.attributes.unwrap().len(), 2);

        let original_id = mood.id.unwrap();
        apply(
            &mut catalog,
            "alter type mood rename to feeling; alter type feeling set schema public;",
            &path,
        );
        assert_eq!(
            catalog
                .type_(Some("public"), "feeling", &path)
                .found()
                .unwrap()
                .id,
            Some(original_id.clone())
        );
        apply(
            &mut catalog,
            "drop type feeling; create type feeling as enum ('new');",
            &path,
        );
        assert_ne!(
            catalog.type_(None, "feeling", &path).found().unwrap().id,
            Some(original_id)
        );
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn overload_mutations_keep_signature_identity_and_invalidation(test_db: PgPool) {
        let path = path(&["public"]);
        let mut catalog = database_catalog(&test_db).await;
        apply(
            &mut catalog,
            "create function overlay_f(a int, b text default 'x', variadic c int[]) returns setof int language sql as 'select 1';
             create function overlay_f(a bigint) returns bigint language sql as 'select 1';
             create function overlay_out(out a int, out b text) returns record language sql as 'select 1, ''x''';",
            &path,
        );
        let Lookup::Found(candidates) = catalog.functions(None, "overlay_f", &path) else {
            panic!("functions missing")
        };
        assert_eq!(candidates.len(), 2);
        let first = candidates.iter().find(|f| f.returns_set).unwrap();
        assert!(first.returns_set);
        let signature = first.signature.as_ref().unwrap();
        assert_eq!(signature.arguments.len(), 3);
        assert_eq!(signature.input_defaults, 1);
        assert_eq!(
            signature.variadic_element,
            Some(type_id(&catalog, "int4", &path))
        );
        apply(
            &mut catalog,
            "create or replace function overlay_f(a int, b text default 'y', variadic c int[]) returns setof int language sql as 'select 1';
             drop function overlay_f(bigint);
             alter function overlay_f(int, text, variadic int[]) rename to renamed_f;",
            &path,
        );
        assert!(matches!(
            catalog.functions(None, "overlay_f", &path),
            Lookup::Missing
        ));
        let Lookup::Found(renamed) = catalog.functions(None, "renamed_f", &path) else {
            panic!("renamed overload missing")
        };
        assert_eq!(renamed.len(), 1);
        assert_eq!(renamed[0].signature.as_ref().unwrap().arguments.len(), 3);
        let out = catalog
            .functions(None, "overlay_out", &path)
            .found()
            .unwrap();
        assert_eq!(out[0].return_columns.as_ref().unwrap().len(), 2);
        assert!(out[0].signature.as_ref().unwrap().return_type.is_none());
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn derived_columns_and_ddl_invalidation_are_typed(test_db: PgPool) {
        let path = path(&["public"]);
        let mut catalog = database_catalog(&test_db).await;
        apply(
            &mut catalog,
            "create table typed_source (id int, name text);
             create view typed_view as select id, name from typed_source;
             create table typed_ctas as select 1 as a, 'x'::text as b;
             create cast (int as text) with inout;
             create operator public.## (leftarg = int, rightarg = int, function = int4pl);",
            &path,
        );
        let source = catalog
            .relation(None, "typed_source", &path)
            .found()
            .unwrap();
        let view = catalog.relation(None, "typed_view", &path).found().unwrap();
        let source_columns = source.columns.unwrap();
        let view_columns = view.columns.unwrap();
        assert_eq!(view_columns[0].ty, source_columns[0].ty);
        assert_eq!(view_columns[1].ty, source_columns[1].ty);
        assert_eq!(
            column_type(&catalog, "typed_ctas", "a", &path),
            Some(Type::Named(type_id(&catalog, "int4", &path)))
        );
        assert_eq!(
            column_type(&catalog, "typed_ctas", "b", &path),
            Some(Type::Named(type_id(&catalog, "text", &path)))
        );
        assert!(matches!(
            catalog.cast(
                &type_id(&catalog, "int4", &path),
                &type_id(&catalog, "text", &path)
            ),
            Lookup::Unknown
        ));
        assert!(
            !catalog
                .operator_candidates(
                    Some("public"),
                    "##",
                    crate::typing::OperatorKind::Infix,
                    &path
                )
                .complete
        );
        apply(&mut catalog, "do $$ begin null; end $$;", &path);
        assert!(catalog.is_tainted());
        assert!(!catalog.function_candidates(None, "abs", &path).complete);
    }
}
