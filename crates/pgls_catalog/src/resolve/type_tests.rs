use std::sync::Arc;

use crate::{Catalog, CatalogBase, Snapshot, typing::format_type};
use sqlx::PgPool;

fn output_types(catalog: &mut Catalog, setup_sql: &str, query: &str) -> Vec<Option<String>> {
    let search_path = vec!["public".to_owned()];
    if !setup_sql.is_empty() {
        for stmt in pgls_query::parse(setup_sql)
            .expect("valid setup SQL")
            .stmts()
        {
            catalog.apply(stmt, &search_path);
        }
    }
    let parsed = pgls_query::parse(query).expect("valid query");
    let root = pgls_query::protobuf::Node {
        node: parsed.stmts().first().map(|node| (**node).clone()),
    };
    super::query_output_types(&root, catalog, &search_path)
        .unwrap_or_else(|| panic!("unknown query shape: {query}"))
        .into_iter()
        .map(|column| column.ty.as_ref().and_then(|ty| format_type(catalog, ty)))
        .collect()
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn infers_query_output_types(test_db: PgPool) {
    let snapshot = Arc::new(Snapshot::load(&test_db).await.unwrap());
    let mut catalog = Catalog::new(Some(Arc::new(CatalogBase::new(snapshot))));
    let cases = [
        (
            "create table typed_values (i int4, big int8, n numeric, t text, b bool, d date, a int4[]);",
            "select i, t as label, * from typed_values",
            vec![
                "integer",
                "text",
                "integer",
                "bigint",
                "numeric",
                "text",
                "boolean",
                "date",
                "integer[]",
            ],
        ),
        (
            "create table typed_values (i int4, big int8, n numeric, t text, b bool, d date, a int4[]);",
            "select v.i, v.* from typed_values v",
            vec![
                "integer",
                "integer",
                "bigint",
                "numeric",
                "text",
                "boolean",
                "date",
                "integer[]",
            ],
        ),
        ("", "values (1), (2.5)", vec!["numeric"]),
        ("", "select 1 union select 2::int8", vec!["bigint"]),
        (
            "",
            "with c as (select 1 as i) select i from c",
            vec!["integer"],
        ),
        (
            "create table typed_values (i int4, t text);",
            "select * from typed_values x join typed_values y using (i)",
            vec!["integer", "text", "text"],
        ),
        ("", "select 1 as x from (values (1)) v", vec!["integer"]),
        ("", "select 1 as x from (select 1) s", vec!["integer"]),
        // Unknown literals remain `unknown` in the static parse-time result. PostgreSQL's
        // protocol Describe output coerces a top-level unknown target to text later.
        (
            "",
            "select 1, 3000000000, 1.5, 'a', true, null",
            vec![
                "integer", "bigint", "numeric", "unknown", "boolean", "unknown",
            ],
        ),
        (
            "",
            "select '1'::int4, now(), current_date",
            vec!["integer", "timestamp with time zone", "date"],
        ),
        (
            "",
            "select coalesce(1, 2.5), nullif(1, 2.5), case when true then 1 else 2.5 end",
            vec!["numeric", "numeric", "numeric"],
        ),
        ("", "select array[1,2]", vec!["integer[]"]),
        (
            "",
            "select (select 1), 1 in (1,2)",
            vec!["integer", "boolean"],
        ),
        (
            "",
            "select count(*), sum(i) from (values (1)) x(i)",
            vec!["bigint", "bigint"],
        ),
        (
            "",
            "select * from unnest(array[1,2]) with ordinality as u(x, n)",
            vec!["integer", "bigint"],
        ),
        (
            "create table typed_returning (i int4);",
            "insert into typed_returning values (1) returning i",
            vec!["integer"],
        ),
    ];
    for (setup, query, expected) in cases {
        assert_eq!(
            output_types(&mut catalog, setup, query),
            expected
                .into_iter()
                .map(|s| Some(s.to_owned()))
                .collect::<Vec<_>>(),
            "{query}"
        );
    }
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn infers_length_result_type(test_db: PgPool) {
    let snapshot = Arc::new(Snapshot::load(&test_db).await.unwrap());
    let mut catalog = Catalog::new(Some(Arc::new(CatalogBase::new(snapshot))));
    assert_eq!(
        output_types(&mut catalog, "", "select length('a')"),
        [Some("integer".into())]
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn infers_mixed_numeric_operator(test_db: PgPool) {
    let snapshot = Arc::new(Snapshot::load(&test_db).await.unwrap());
    let mut catalog = Catalog::new(Some(Arc::new(CatalogBase::new(snapshot))));
    assert_eq!(
        output_types(&mut catalog, "", "select 1 + 2.5"),
        [Some("numeric".into())]
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn infers_timestamp_minus_interval(test_db: PgPool) {
    let snapshot = Arc::new(Snapshot::load(&test_db).await.unwrap());
    let mut catalog = Catalog::new(Some(Arc::new(CatalogBase::new(snapshot))));
    assert_eq!(
        output_types(&mut catalog, "", "select now() - interval '1 day'"),
        [Some("timestamp with time zone".into())]
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn infers_array_subscripts(test_db: PgPool) {
    let snapshot = Arc::new(Snapshot::load(&test_db).await.unwrap());
    let mut catalog = Catalog::new(Some(Arc::new(CatalogBase::new(snapshot))));
    assert_eq!(
        output_types(
            &mut catalog,
            "",
            "select (array[1,2])[1], (array[1,2])[1:2]"
        ),
        [Some("integer".into()), Some("integer[]".into())]
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn infers_array_agg_result(test_db: PgPool) {
    let snapshot = Arc::new(Snapshot::load(&test_db).await.unwrap());
    let mut catalog = Catalog::new(Some(Arc::new(CatalogBase::new(snapshot))));
    assert_eq!(
        output_types(
            &mut catalog,
            "",
            "select array_agg(i) from (values (1)) x(i)"
        ),
        [Some("integer[]".into())]
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn unknown_output_types_stay_unknown(test_db: PgPool) {
    let snapshot = Arc::new(Snapshot::load(&test_db).await.unwrap());
    let catalog = Catalog::new(Some(Arc::new(CatalogBase::new(snapshot))));
    for query in [
        "with recursive x(n) as (select 1 union all select n + 1 from x) select n from x",
        "select missing from (values (1)) x",
        "select absent_function(1)",
    ] {
        let search_path = vec!["public".to_owned()];
        let parsed = pgls_query::parse(query).unwrap();
        let root = pgls_query::protobuf::Node {
            node: parsed.stmts().first().map(|node| (**node).clone()),
        };
        if let Some(columns) = super::query_output_types(&root, &catalog, &search_path) {
            assert!(
                columns.iter().all(|column| column.ty.is_none()),
                "{query}: {columns:?}"
            );
        }
    }
}
