use pgls_pretty_print::{
    CommaStyle, FormatConfig, Layout, format_statement,
    renderer::{IndentStyle, KeywordCase},
};

fn format_idempotently(sql: &str, config: &FormatConfig) -> String {
    let ast = pgls_query::parse(sql).unwrap().into_root().unwrap();
    let first = format_statement(&ast, sql, config).unwrap().formatted;
    let ast = pgls_query::parse(&first).unwrap().into_root().unwrap();
    let second = format_statement(&ast, &first, config).unwrap().formatted;
    assert_eq!(first, second);
    first
}

#[test]
fn values_rows_fit_on_one_line_by_default() {
    for comma_style in [CommaStyle::Trailing, CommaStyle::Leading] {
        let config = FormatConfig {
            comma_style,
            ..Default::default()
        };
        assert_eq!(
            format_idempotently("VALUES (1, 'one'), (2, 'two');", &config),
            "values (1, 'one'), (2, 'two');"
        );
    }
}

#[test]
fn expanded_values_rows_use_leading_commas_and_tabs() {
    let config = FormatConfig {
        values_row_layout: Layout::Expanded,
        comma_style: CommaStyle::Leading,
        indent_style: IndentStyle::Tabs,
        indent_size: 4,
        keyword_case: KeywordCase::Upper,
        isolate_semicolon: true,
        ..Default::default()
    };
    let expected = "VALUES\n\t('MONTHLY', 1)\n\t, ('QUARTERLY', 3)\n\t, ('HALF_YEAR', 6)\n;";
    for sql in [
        "VALUES ('MONTHLY', 1), ('QUARTERLY', 3), ('HALF_YEAR', 6);",
        "VALUES\n('MONTHLY', 1),\n('QUARTERLY', 3),\n('HALF_YEAR', 6);",
    ] {
        assert_eq!(format_idempotently(sql, &config), expected);
    }
}

#[test]
fn expanded_values_rows_can_use_trailing_commas() {
    let config = FormatConfig {
        values_row_layout: Layout::Expanded,
        ..Default::default()
    };
    assert_eq!(
        format_idempotently("VALUES (1, 'one'), (2, 'two');", &config),
        "values\n  (1, 'one'),\n  (2, 'two');"
    );
}

#[test]
fn expanded_values_indents_even_a_single_row() {
    let config = FormatConfig {
        values_row_layout: Layout::Expanded,
        ..Default::default()
    };
    assert_eq!(
        format_idempotently("VALUES (1);", &config),
        "values\n  (1);"
    );
}

#[test]
fn fit_values_rows_respect_leading_commas_when_wrapping() {
    let config = FormatConfig {
        comma_style: CommaStyle::Leading,
        line_width: 80,
        ..Default::default()
    };
    assert_eq!(
        format_idempotently(
            "VALUES ('a moderately long value for the first row', 1), \
             ('a moderately long value for the second row', 2);",
            &config
        ),
        "values\n  ('a moderately long value for the first row', 1)\n  \
         , ('a moderately long value for the second row', 2);"
    );
}

#[test]
fn expanded_values_rows_still_wrap_long_tuples() {
    let config = FormatConfig {
        values_row_layout: Layout::Expanded,
        comma_style: CommaStyle::Leading,
        line_width: 80,
        ..Default::default()
    };
    let output = format_idempotently(
        "VALUES ('a moderately long value for the first column', \
         'a moderately long value for the second column'), (1, 2);",
        &config,
    );
    assert!(output.contains("\n    , 'a moderately long value for the second column'"));
    assert!(output.contains("\n  , (1, 2)"));
    assert!(output.lines().all(|line| line.len() <= 80), "{output}");
}

#[test]
fn values_layout_applies_to_insert_and_nested_values() {
    let config = FormatConfig {
        values_row_layout: Layout::Expanded,
        comma_style: CommaStyle::Leading,
        ..Default::default()
    };
    for sql in [
        "INSERT INTO frequencies (name, months) VALUES ('MONTHLY', 1), ('QUARTERLY', 3);",
        "SELECT * FROM (VALUES ('MONTHLY', 1), ('QUARTERLY', 3)) AS frequencies(name, months);",
        "WITH frequencies AS (VALUES ('MONTHLY', 1), ('QUARTERLY', 3)) SELECT * FROM frequencies;",
        "INSERT INTO frequencies VALUES ('MONTHLY', 1), ('QUARTERLY', 3) ON CONFLICT DO NOTHING;",
    ] {
        let output = format_idempotently(sql, &config);
        assert!(output.contains("values\n"), "{output}");
        assert!(output.contains(", ('QUARTERLY', 3)"), "{output}");
        assert!(!output.contains("('MONTHLY', 1),"), "{output}");
    }
}

#[test]
fn expanded_values_rows_preserve_comments() {
    let config = FormatConfig {
        values_row_layout: Layout::Expanded,
        comma_style: CommaStyle::Leading,
        ..Default::default()
    };
    for (sql, comment) in [
        (
            "VALUES (1, 'one'), -- first row\n(2, 'two');",
            "-- first row",
        ),
        (
            "VALUES (1, 'one'),\n-- second row\n(2, 'two');",
            "-- second row",
        ),
        (
            "VALUES (1 /* first value */, 'one'), (2, 'two');",
            "/* first value */",
        ),
    ] {
        let output = format_idempotently(sql, &config);
        assert_eq!(output.matches(comment).count(), 1, "{output}");
    }
}
