use std::sync::Arc;

use pgls_catalog::{Catalog, CatalogBase, Snapshot};
use pgls_inlay_hints::{InlayHintKind, InlayHintsParams, inlay_hints};
use sqlx::{Executor, PgPool};

const SETUP: &str = r#"
    create function make_point(x int, y int) returns int language sql as 'select x + y';
    create function add_unnamed(int, int) returns int language sql as 'select $1 + $2';
    create function half_named(int, y int) returns int language sql as 'select $1 + y';
    create function sum_all(base int, variadic nums int[]) returns int
        language sql as 'select base';
    create function greet(name text, greeting text default 'hi') returns text
        language sql as 'select greeting || name';
    create function amb(a int, b text) returns int language sql as 'select 1';
    create function amb(c text, d int) returns int language sql as 'select 1';
    create table points (x int, y int);
    create procedure move_point(x int, out moved int, inout y int)
        language plpgsql as $$ begin moved := x; end $$;
    create function measure(größe int, ö int) returns int language sql as 'select 1';
    create table sizes (größe int, Ö int);
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

#[pgls_test_utils::db_test]
async fn positional_arguments(test_db: PgPool) {
    let f = fixture(&test_db).await;
    assert_eq!(
        f.hinted("select make_point(1, 2 + 3)"),
        "select make_point(x: 1, y: 2 + 3)"
    );
}

#[pgls_test_utils::db_test]
async fn nested_calls_and_from_functions(test_db: PgPool) {
    let f = fixture(&test_db).await;
    assert_eq!(
        f.hinted("select * from make_point(1, public.make_point((1), 2 /* y */))"),
        "select * from make_point(x: 1, y: public.make_point(x: (1), y: 2 /* y */))"
    );
}

#[pgls_test_utils::db_test]
async fn named_arguments_get_no_hints(test_db: PgPool) {
    let f = fixture(&test_db).await;
    assert_eq!(
        f.hinted("select make_point(y => 1, x => 2)"),
        "select make_point(y => 1, x => 2)"
    );
}

#[pgls_test_utils::db_test]
async fn mixed_calls_hint_the_positional_arguments(test_db: PgPool) {
    let f = fixture(&test_db).await;
    assert_eq!(
        f.hinted("select make_point(1, y => 2)"),
        "select make_point(x: 1, y => 2)"
    );
}

#[pgls_test_utils::db_test]
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

/// Postgres only folds ASCII letters in unquoted identifiers, so `Ö` is not `ö`.
#[pgls_test_utils::db_test]
async fn non_ascii_arguments_named_like_the_parameter(test_db: PgPool) {
    let f = fixture(&test_db).await;
    assert_eq!(
        f.hinted("select measure(größe, Ö) from sizes"),
        "select measure(größe, ö: Ö) from sizes"
    );
}

#[pgls_test_utils::db_test]
async fn unnamed_parameters_get_no_hints(test_db: PgPool) {
    let f = fixture(&test_db).await;
    assert_eq!(
        f.hinted("select add_unnamed(1, 2)"),
        "select add_unnamed(1, 2)"
    );
}

#[pgls_test_utils::db_test]
async fn only_named_parameters_get_hints(test_db: PgPool) {
    let f = fixture(&test_db).await;
    assert_eq!(
        f.hinted("select half_named(1, 2)"),
        "select half_named(1, y: 2)"
    );
}

#[pgls_test_utils::db_test]
async fn calls_in_a_long_statement(test_db: PgPool) {
    let f = fixture(&test_db).await;
    let items = |hinted: bool| {
        (0..500)
            .map(|i| {
                if hinted {
                    format!("abs({i}) + make_point(x: {i}, y: abs({i}))")
                } else {
                    format!("abs({i}) + make_point({i}, abs({i}))")
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    assert_eq!(
        f.hinted(&format!("select {}", items(false))),
        format!("select {}", items(true))
    );
}

#[pgls_test_utils::db_test]
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

#[pgls_test_utils::db_test]
async fn default_arguments(test_db: PgPool) {
    let f = fixture(&test_db).await;
    assert_eq!(f.hinted("select greet('a')"), "select greet(name: 'a')");
}

#[pgls_test_utils::db_test]
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

#[pgls_test_utils::db_test]
async fn sql_syntax_gets_no_hints(test_db: PgPool) {
    let f = fixture(&test_db).await;
    assert_eq!(
        f.hinted("select extract(year from now())"),
        "select extract(year from now())"
    );
}

#[pgls_test_utils::db_test]
async fn procedure_calls_pass_output_arguments(test_db: PgPool) {
    let f = fixture(&test_db).await;
    assert_eq!(
        f.hinted("call move_point(1, null, 2)"),
        "call move_point(x: 1, moved: null, y: 2)"
    );
}

#[pgls_test_utils::db_test]
async fn only_ascii_letters_of_unquoted_arguments_are_downcased(test_db: PgPool) {
    test_db
        .execute(r#"create function accent("Éa" int) returns int language sql as 'select 1'"#)
        .await
        .expect("Failed to create the function");
    let f = fixture(&test_db).await;
    // Like Postgres in a UTF8 database, `ÉA` stands for `Éa`, not `éa`.
    assert_eq!(
        f.hinted(r#"select accent(t.ÉA) from (select 1 as "Éa") t"#),
        r#"select accent(t.ÉA) from (select 1 as "Éa") t"#
    );
}
