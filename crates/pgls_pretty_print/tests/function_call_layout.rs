use pgls_pretty_print::{FormatConfig, format_statement};

fn format_idempotently(sql: &str, config: &FormatConfig) -> String {
    let ast = pgls_query::parse(sql).unwrap().into_root().unwrap();
    let first = format_statement(&ast, sql, config).unwrap().formatted;
    let ast = pgls_query::parse(&first).unwrap().into_root().unwrap();
    let second = format_statement(&ast, &first, config).unwrap().formatted;
    assert_eq!(first, second);
    first
}

#[test]
fn count_star_stays_together_when_the_filter_breaks() {
    let output = format_idempotently(
        "select count(*) filter (where status = 'active' and created_at > now() - interval '30 days' and deleted_at is null) from users",
        &FormatConfig::default(),
    );
    assert!(output.contains("  count(*)\n    filter (where"), "{output}");
}

#[test]
fn within_group_continues_one_level_in() {
    let config = FormatConfig {
        line_width: 60,
        ..Default::default()
    };
    let output = format_idempotently(
        "select percentile_cont(0.5) within group (order by some_long_column_name, another_long_column) from t",
        &config,
    );
    assert!(
        output.contains("  percentile_cont(0.5)\n    within group ("),
        "{output}"
    );
}

#[test]
fn long_array_literal_fills_indented_lines() {
    let output = format_idempotently(
        "select * from t where t.a = any (array[1111111111, 2222222222, 3333333333, 4444444444, 5555555555, 6666666666, 7777777777, 8888888888])",
        &FormatConfig {
            line_width: 60,
            ..Default::default()
        },
    );
    assert_eq!(
        output,
        "select\n  *\nfrom\n  t\nwhere\n  t.a = any (array[\n    1111111111, 2222222222, 3333333333, 4444444444,\n    5555555555, 6666666666, 7777777777, 8888888888\n  ]);"
    );
}

#[test]
fn short_array_literal_stays_on_one_line() {
    assert_eq!(
        format_idempotently("select array[1, 2, 3]", &FormatConfig::default()),
        "select array[1, 2, 3];"
    );
}

#[test]
fn json_value_respects_the_line_width() {
    let config = FormatConfig {
        line_width: 60,
        ..Default::default()
    };
    let output = format_idempotently(
        "select json_value(jsonb_column_with_long_name, '$.some.very.long.path.expression.here' returning integer default 0 on empty error on error) from t",
        &config,
    );
    assert_eq!(
        output,
        "select\n  json_value(\n    jsonb_column_with_long_name,\n    '$.some.very.long.path.expression.here'\n    returning int\n    default 0 on empty\n    error on error\n  )\nfrom\n  t;"
    );
}

#[test]
fn json_query_and_json_exists_respect_the_line_width() {
    let config = FormatConfig {
        line_width: 60,
        ..Default::default()
    };
    let output = format_idempotently(
        "select json_query(jsonb_column_with_long_name, '$.some.path' returning jsonb with conditional wrapper keep quotes) from t",
        &config,
    );
    assert!(output.lines().all(|line| line.len() <= 60), "{output}");
    assert!(
        output.contains("\n    with conditional wrapper\n"),
        "{output}"
    );

    let output = format_idempotently(
        "select json_exists(jsonb_column_with_long_name, '$.some.very.long.path.expression' error on error) from t",
        &config,
    );
    assert!(output.lines().all(|line| line.len() <= 60), "{output}");
    assert!(output.contains("\n    error on error\n  )"), "{output}");

    // A call that fits stays on one line.
    assert_eq!(
        format_idempotently("select json_value(j, '$.a' returning int)", &config),
        "select json_value(j, '$.a' returning int);"
    );
}
