mod common;

use pgls_pretty_print::{CommaStyle, FormatConfig, Layout, format_statement};

fn format_idempotently(sql: &str, config: &FormatConfig) -> String {
    let ast = pgls_query::parse(sql).unwrap().into_root().unwrap();
    let first = format_statement(&ast, sql, config).unwrap().formatted;

    let ast = pgls_query::parse(&first).unwrap().into_root().unwrap();
    let second = format_statement(&ast, &first, config).unwrap().formatted;
    assert_eq!(second, first);
    common::assert_clean_whitespace(&first);
    first
}

#[test]
fn leading_commas_do_not_add_a_space_before_the_comma() {
    let config = FormatConfig {
        comma_style: CommaStyle::Leading,
        ..Default::default()
    };
    assert_eq!(
        format_idempotently("select a, b from t", &config),
        "select a, b from t;"
    );
    assert_eq!(
        format_idempotently("insert into t (a, b) values (1, 2)", &config),
        "insert into t (a, b) values (1, 2);"
    );

    // A list that breaks still puts the comma in front of the item.
    let config = FormatConfig {
        line_width: 40,
        ..config
    };
    assert_eq!(
        format_idempotently(
            "select first_column, second_column, third_column from t",
            &config
        ),
        "select\n  first_column\n  , second_column\n  , third_column\nfrom\n  t;"
    );
}

#[test]
fn leading_commas_in_expanded_layout_break_before_every_comma() {
    let config = FormatConfig {
        comma_style: CommaStyle::Leading,
        layout: Layout::Expanded,
        ..Default::default()
    };
    assert_eq!(
        format_idempotently("select a, b from t", &config),
        "select\n  a\n  , b\nfrom\n  t;"
    );
}

#[test]
fn no_trailing_whitespace_after_a_block_comment_in_a_policy() {
    let output = format_idempotently(
        "create policy p on t using (\n  a = 1 -- first\n  and b = 2 /* second */\n);",
        &FormatConfig::default(),
    );
    assert!(output.contains("b = 2 /* second */\n);"), "{output}");
}

#[test]
fn no_trailing_whitespace_before_an_isolated_semicolon() {
    let config = FormatConfig {
        line_width: 40,
        isolate_semicolon: true,
        ..Default::default()
    };
    let output = format_idempotently(
        "create foreign data wrapper foo_wrapper handler foo_handler validator bar_validator",
        &config,
    );
    assert!(output.ends_with("validator bar_validator\n;"), "{output}");
}

#[test]
fn no_space_inside_parentheses() {
    let config = FormatConfig::default();
    assert_eq!(
        format_idempotently("select * from a join b using (x, y)", &config),
        "select * from a inner join b using (x, y);"
    );
    assert_eq!(
        format_idempotently("create foreign table ft (a int) server s", &config),
        "create foreign table ft (a int) server s;"
    );
    assert_eq!(
        format_idempotently(
            "select * from json_table('{}'::jsonb, '$' as a columns (c int))",
            &config
        ),
        "select * from json_table(cast('{}' as jsonb), '$' as a columns (c int));"
    );
    assert_eq!(
        format_idempotently(
            "select * from xmltable('/root' passing '<root/>' columns item text)",
            &config
        ),
        "select * from xmltable('/root' passing '<root/>' columns item text);"
    );
    assert_eq!(
        format_idempotently(
            "select sum(x) over (rows between 1 preceding and current row) from t",
            &config
        ),
        "select sum(x) over (rows between 1 preceding and current row) from t;"
    );
    assert_eq!(
        format_idempotently("alter publication p add table t where (a > 1)", &config),
        "alter publication p add table t where (a > 1);"
    );
    assert_eq!(
        format_idempotently(
            "create table t of person_type (id with options not null)",
            &config
        ),
        "create table t of person_type (id with options not null);"
    );
}

#[test]
fn no_blank_line_in_empty_parentheses() {
    let config = FormatConfig {
        line_width: 40,
        ..Default::default()
    };
    assert_eq!(
        format_idempotently(
            "create table i_am_a_failure () using i_do_not_exist_am",
            &config
        ),
        "create table i_am_a_failure () using i_do_not_exist_am;"
    );

    let output = format_idempotently(
        "select sum(salary) over (range between unbounded preceding and current row) from empsalary",
        &config,
    );
    assert!(output.contains("over (\n"), "{output}");
}
