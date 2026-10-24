use std::sync::Arc;

use pgls_catalog::{Catalog, CatalogBase, Snapshot};
use pgls_inlay_hints::{InlayHintKind, InlayHintsParams, inlay_hints};
use sqlx::{Executor, PgPool};

const SETUP: &str = r#"
    create function make_point(x int, y int) returns int language sql as 'select x + y';
    create function add_unnamed(int, int) returns int language sql as 'select $1 + $2';
    create function sum_all(base int, variadic nums int[]) returns int
        language sql as 'select base';
    create function greet(name text, greeting text default 'hi') returns text
        language sql as 'select greeting || name';
    create function amb(a int, b text) returns int language sql as 'select 1';
    create function amb(c text, d int) returns int language sql as 'select 1';
    create table points (x int, y int);
    create procedure move_point(x int, out moved int, inout y int)
        language plpgsql as $$ begin moved := x; end $$;
"#;

struct Fixture {
    catalog: Catalog,
    search_path: Vec<String>,
}

async fn fixture(test_db: &PgPool) -> Fixture {
    test_db
        .execute(SETUP)
        .await
        .expect("Failed to setup test database");
    let snapshot = Snapshot::load(test_db)
        .await
        .expect("Failed to load snapshot");
    Fixture {
        search_path: snapshot.expand_search_path(&["public".to_string()]),
        catalog: Catalog::new(Some(Arc::new(CatalogBase::new(Arc::new(snapshot))))),
    }
}

impl Fixture {
    /// The statement with each hint inserted before its argument.
    fn hinted(&self, sql: &str) -> String {
        let stmt = pgls_query::parse(sql)
            .expect("Failed to parse")
            .into_root()
            .expect("Expected a statement");

        let hints = inlay_hints(InlayHintsParams {
            stmt: &stmt,
            sql,
            catalog: &self.catalog,
            search_path: &self.search_path,
            function: None,
        });

        let mut result = sql.to_string();
        for hint in hints.iter().rev() {
            assert_eq!(hint.kind, InlayHintKind::Parameter);
            result.insert_str(usize::from(hint.offset), &format!("{} ", hint.label));
        }
        result
    }
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn positional_arguments(test_db: PgPool) {
    let f = fixture(&test_db).await;
    assert_eq!(
        f.hinted("select make_point(1, 2 + 3)"),
        "select make_point(x: 1, y: 2 + 3)"
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn nested_calls_and_from_functions(test_db: PgPool) {
    let f = fixture(&test_db).await;
    assert_eq!(
        f.hinted("select * from make_point(1, public.make_point((1), 2 /* y */))"),
        "select * from make_point(x: 1, y: public.make_point(x: (1), y: 2 /* y */))"
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn named_arguments_get_no_hints(test_db: PgPool) {
    let f = fixture(&test_db).await;
    assert_eq!(
        f.hinted("select make_point(y => 1, x => 2)"),
        "select make_point(y => 1, x => 2)"
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn mixed_calls_hint_the_positional_arguments(test_db: PgPool) {
    let f = fixture(&test_db).await;
    assert_eq!(
        f.hinted("select make_point(1, y => 2)"),
        "select make_point(x: 1, y => 2)"
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn arguments_named_like_the_parameter_get_no_hints(test_db: PgPool) {
    let f = fixture(&test_db).await;
    assert_eq!(
        f.hinted("select make_point(p.y, p.x) from points p"),
        "select make_point(x: p.y, y: p.x) from points p"
    );
    assert_eq!(
        f.hinted("select make_point(p.x, \"y\") from points p"),
        "select make_point(p.x, \"y\") from points p"
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn unnamed_parameters_get_no_hints(test_db: PgPool) {
    let f = fixture(&test_db).await;
    assert_eq!(
        f.hinted("select add_unnamed(1, 2)"),
        "select add_unnamed(1, 2)"
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn variadic_arguments(test_db: PgPool) {
    let f = fixture(&test_db).await;
    assert_eq!(
        f.hinted("select sum_all(1, 2, 3)"),
        "select sum_all(base: 1, nums: 2, 3)"
    );
    assert_eq!(
        f.hinted("select sum_all(1, variadic array[2, 3])"),
        "select sum_all(base: 1, nums: variadic array[2, 3])"
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn default_arguments(test_db: PgPool) {
    let f = fixture(&test_db).await;
    assert_eq!(f.hinted("select greet('a')"), "select greet(name: 'a')");
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn ambiguous_and_unknown_calls_get_no_hints(test_db: PgPool) {
    let f = fixture(&test_db).await;
    assert_eq!(f.hinted("select amb('1', '2')"), "select amb('1', '2')");
    assert_eq!(
        f.hinted("select make_point(a, 1) from unknown_table"),
        "select make_point(a, 1) from unknown_table"
    );
    assert_eq!(
        f.hinted("select unknown_function(1)"),
        "select unknown_function(1)"
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn sql_syntax_gets_no_hints(test_db: PgPool) {
    let f = fixture(&test_db).await;
    assert_eq!(
        f.hinted("select extract(year from now())"),
        "select extract(year from now())"
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn procedure_calls_pass_output_arguments(test_db: PgPool) {
    let f = fixture(&test_db).await;
    assert_eq!(
        f.hinted("call move_point(1, null, 2)"),
        "call move_point(x: 1, moved: null, y: 2)"
    );
}
