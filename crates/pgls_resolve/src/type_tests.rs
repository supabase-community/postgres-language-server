use std::sync::Arc;

use pgls_catalog::{Catalog, Snapshot, typing::format_type};
use sqlx::PgPool;

fn output_types(catalog: &mut Catalog, setup_sql: &str, query: &str) -> Vec<Option<String>> {
    let search_path = vec!["public".to_owned()];
    if !setup_sql.is_empty() {
        for stmt in pgls_query::parse(setup_sql)
            .expect("valid setup SQL")
            .stmts()
        {
            crate::apply_resolved(catalog, stmt, &search_path);
        }
    }
    let parsed = pgls_query::parse(query).expect("valid query");
    let root = pgls_query::protobuf::Node {
        node: parsed.stmts().first().map(|node| (**node).clone()),
    };
    super::query_columns(&root, catalog, &search_path)
        .unwrap_or_else(|| panic!("unknown query shape: {query}"))
        .into_iter()
        .map(|column| column.ty.as_ref().and_then(|ty| format_type(catalog, ty)))
        .collect()
}

#[pgls_test_utils::db_test]
async fn infers_query_columns(test_db: PgPool) {
    let snapshot = Arc::new(Snapshot::load(&test_db).await.unwrap());
    let mut catalog = Catalog::from_snapshot(snapshot);
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
            vec!["integer", "bigint", "numeric", "text", "boolean", "text"],
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

#[pgls_test_utils::db_test]
async fn infers_fields_of_a_composite_without_its_kind(test_db: PgPool) {
    // `typtype` is optional in a snapshot's JSON; the attributes still give the fields.
    sqlx::raw_sql("create type pair as (a integer, b text); create table items (p pair);")
        .execute(&test_db)
        .await
        .unwrap();
    let mut snapshot = Snapshot::load(&test_db).await.unwrap();
    let pair = snapshot
        .types
        .iter_mut()
        .find(|ty| ty.schema == "public" && ty.name == "pair")
        .expect("the composite type pair");
    pair.typtype = Default::default();
    let mut catalog = Catalog::from_snapshot(snapshot);
    assert_eq!(
        output_types(&mut catalog, "", "select (p).a from items"),
        [Some("integer".into())]
    );
}

#[pgls_test_utils::db_test]
async fn infers_length_result_type(test_db: PgPool) {
    let snapshot = Arc::new(Snapshot::load(&test_db).await.unwrap());
    let mut catalog = Catalog::from_snapshot(snapshot);
    assert_eq!(
        output_types(&mut catalog, "", "select length('a')"),
        [Some("integer".into())]
    );
}

#[pgls_test_utils::db_test]
async fn infers_mixed_numeric_operator(test_db: PgPool) {
    let snapshot = Arc::new(Snapshot::load(&test_db).await.unwrap());
    let mut catalog = Catalog::from_snapshot(snapshot);
    assert_eq!(
        output_types(&mut catalog, "", "select 1 + 2.5"),
        [Some("numeric".into())]
    );
}

#[pgls_test_utils::db_test]
async fn infers_timestamp_minus_interval(test_db: PgPool) {
    let snapshot = Arc::new(Snapshot::load(&test_db).await.unwrap());
    let mut catalog = Catalog::from_snapshot(snapshot);
    assert_eq!(
        output_types(&mut catalog, "", "select now() - interval '1 day'"),
        [Some("timestamp with time zone".into())]
    );
}

#[pgls_test_utils::db_test]
async fn infers_array_subscripts(test_db: PgPool) {
    let snapshot = Arc::new(Snapshot::load(&test_db).await.unwrap());
    let mut catalog = Catalog::from_snapshot(snapshot);
    assert_eq!(
        output_types(
            &mut catalog,
            "",
            "select (array[1,2])[1], (array[1,2])[1:2]"
        ),
        [Some("integer".into()), Some("integer[]".into())]
    );
}

/// A field of a domain over a composite type has the type of the attribute, like for the
/// composite type itself ([`get_expr_result_type`]).
///
/// [`get_expr_result_type`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/utils/fmgr/funcapi.c#L299
#[pgls_test_utils::db_test]
async fn infers_fields_of_domains_over_composite_types(test_db: PgPool) {
    let snapshot = Arc::new(Snapshot::load(&test_db).await.unwrap());
    let mut catalog = Catalog::from_snapshot(snapshot);
    assert_eq!(
        output_types(
            &mut catalog,
            "create type typed_pair as (a int4, b text); \
             create domain typed_pair_domain as typed_pair; \
             create table typed_pairs (p typed_pair, d typed_pair_domain);",
            "select (p).a, (d).a, (d).b from typed_pairs"
        ),
        [
            Some("integer".into()),
            Some("integer".into()),
            Some("text".into())
        ]
    );
}

#[pgls_test_utils::db_test]
async fn infers_array_agg_result(test_db: PgPool) {
    let snapshot = Arc::new(Snapshot::load(&test_db).await.unwrap());
    let mut catalog = Catalog::from_snapshot(snapshot);
    assert_eq!(
        output_types(
            &mut catalog,
            "",
            "select array_agg(i) from (values (1)) x(i)"
        ),
        [Some("integer[]".into())]
    );
}

#[pgls_test_utils::db_test]
async fn infers_the_types_of_recursive_ctes(test_db: PgPool) {
    let snapshot = Arc::new(Snapshot::load(&test_db).await.unwrap());
    let mut catalog = Catalog::from_snapshot(snapshot);
    let types = |catalog: &mut Catalog, query| output_types(catalog, "", query);
    let some = |names: &[&str]| -> Vec<Option<String>> {
        names.iter().map(|name| Some(name.to_string())).collect()
    };
    // The types of the non-recursive term, which the recursive term sees too.
    assert_eq!(
        types(
            &mut catalog,
            "with recursive x(n, s) as (select 1, 'a' union all select n + 1, s || 'b' from x \
             where n < 3) select n, s, n + 1 from x"
        ),
        some(&["integer", "text", "integer"])
    );
    // Without a column list, the CTE is named after the non-recursive term.
    assert_eq!(
        types(
            &mut catalog,
            "with recursive x as (select 1::int8 as n union select n + 1 from x where n < 3) \
             select n from x"
        ),
        some(&["bigint"])
    );
    // A CTE of WITH RECURSIVE that isn't a UNION can't be recursive.
    assert_eq!(
        types(
            &mut catalog,
            "with recursive x as (select 1.5 as n) select n from x"
        ),
        some(&["numeric"])
    );
    // Postgres rejects a recursive CTE whose types differ from those of its non-recursive term,
    // and a UNION that doesn't reference itself has the common type of both. We don't tell them
    // apart.
    assert_eq!(
        types(
            &mut catalog,
            "with recursive x(n) as (select 1 union all select n + 1.5 from x where n < 3) \
             select n from x"
        ),
        [None]
    );
    assert_eq!(
        types(
            &mut catalog,
            "with recursive x(n) as (select 1 union select 2.5) select n from x"
        ),
        [None]
    );
}

#[pgls_test_utils::db_test]
async fn unknown_output_types_stay_unknown(test_db: PgPool) {
    let snapshot = Arc::new(Snapshot::load(&test_db).await.unwrap());
    let catalog = Catalog::from_snapshot(snapshot);
    for query in [
        "select missing from (values (1)) x",
        "select absent_function(1)",
    ] {
        let search_path = vec!["public".to_owned()];
        let parsed = pgls_query::parse(query).unwrap();
        let root = pgls_query::protobuf::Node {
            node: parsed.stmts().first().map(|node| (**node).clone()),
        };
        if let Some(columns) = super::query_columns(&root, &catalog, &search_path) {
            assert!(
                columns.iter().all(|column| column.ty.is_none()),
                "{query}: {columns:?}"
            );
        }
    }
}

#[pgls_test_utils::db_test]
async fn knows_the_types_of_each_select_list_entry(test_db: PgPool) {
    let snapshot = Arc::new(Snapshot::load(&test_db).await.unwrap());
    let mut catalog = Catalog::from_snapshot(snapshot);
    let setup = "create type place as (street text, zip int4); \
        create table places (p place); \
        create function pairs() returns setof record language sql as 'select 1, ''a''';";
    // `(t.c).*` has the fields of the composite type.
    assert_eq!(
        output_types(&mut catalog, setup, "select (p).*, (x.p).zip from places x"),
        [
            Some("text".into()),
            Some("integer".into()),
            Some("integer".into())
        ]
    );
    // Column definitions give the types of a function returning `record`.
    assert_eq!(
        output_types(
            &mut catalog,
            "",
            "select * from pairs() as t(a int, b text)"
        ),
        [Some("integer".into()), Some("text".into())]
    );
    assert_eq!(
        output_types(
            &mut catalog,
            "",
            "select * from rows from (pairs() as (a int8, b text), pairs() as (c bool, d int))"
        ),
        [
            Some("bigint".into()),
            Some("text".into()),
            Some("boolean".into()),
            Some("integer".into())
        ]
    );
    // A `*` whose columns have no types doesn't hide the types of the other entries.
    assert_eq!(
        output_types(
            &mut catalog,
            "",
            "select *, 1 as one from (select 1 as a union select x from nope) s"
        ),
        [None, Some("integer".into())]
    );
}

/// Moved from the catalog's `derived_columns_and_ddl_invalidation_are_typed`: views and
/// `CREATE TABLE AS` get the types of their query's output columns.
#[pgls_test_utils::db_test]
async fn derived_columns_are_typed(test_db: PgPool) {
    use pgls_catalog::typing::Type;

    let snapshot = Arc::new(Snapshot::load(&test_db).await.unwrap());
    let mut catalog = Catalog::from_snapshot(snapshot);
    let path = vec!["public".to_owned()];
    for stmt in pgls_query::parse(
        "create table typed_source (id int, name text);
         create view typed_view as select id, name from typed_source;
         create table typed_ctas as select 1 as a, 'x'::text as b;",
    )
    .expect("valid sql")
    .stmts()
    {
        super::apply_resolved(&mut catalog, stmt, &path);
    }
    let columns = |name: &str| {
        catalog
            .relation(None, name, &path)
            .found()
            .and_then(|relation| relation.columns)
            .unwrap()
    };
    let builtin = |name: &str| {
        catalog
            .type_(Some("pg_catalog"), name, &path)
            .found()
            .and_then(|ty| ty.id)
            .map(Type::Named)
    };
    let (source, view) = (columns("typed_source"), columns("typed_view"));
    assert_eq!(view[0].ty, source[0].ty);
    assert_eq!(view[1].ty, source[1].ty);
    let ctas = columns("typed_ctas");
    assert_eq!(ctas[0].ty, builtin("int4"));
    assert_eq!(ctas[1].ty, builtin("text"));
}
