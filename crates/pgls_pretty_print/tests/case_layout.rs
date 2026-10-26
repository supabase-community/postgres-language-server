use pgls_pretty_print::{
    ClauseBodyStyle, CommaStyle, FormatConfig, Layout, LogicalOperatorPlacement, format_statement,
    renderer::{IndentStyle, KeywordCase},
};

fn expanded_config() -> FormatConfig {
    FormatConfig {
        line_width: 200,
        indent_size: 4,
        indent_style: IndentStyle::Tabs,
        keyword_case: KeywordCase::Upper,
        comma_style: CommaStyle::Leading,
        logical_operator_placement: LogicalOperatorPlacement::Leading,
        layout: Layout::Expanded,
        isolate_semicolon: true,
        ..Default::default()
    }
}

fn format_idempotently(sql: &str, config: &FormatConfig) -> String {
    let ast = pgls_query::parse(sql).unwrap().into_root().unwrap();
    let first = format_statement(&ast, sql, config).unwrap().formatted;
    let ast = pgls_query::parse(&first).unwrap().into_root().unwrap();
    let second = format_statement(&ast, &first, config).unwrap().formatted;
    assert_eq!(first, second);
    first
}

fn assert_expanded_layout(sql: &str, expected: &str) {
    for line_width in [80, 200] {
        for indent_style in [IndentStyle::Tabs, IndentStyle::Spaces] {
            let expected = match indent_style {
                IndentStyle::Tabs => expected.to_string(),
                IndentStyle::Spaces => expected.replace('\t', "    "),
            };
            let config = FormatConfig {
                line_width,
                indent_style,
                ..expanded_config()
            };
            assert_eq!(format_idempotently(sql, &config), expected);
        }
    }
}

#[test]
fn expanded_simple_case_breaks_before_every_when() {
    let sql = "SELECT CASE payment_term WHEN 'DEPOSIT' THEN 'ADVANCE' \
               WHEN 'DUE' THEN 'ARREARS' ELSE 'OTHER' END AS payment_type;";
    let expected = "SELECT\n\
                    \tCASE payment_term\n\
                    \t\tWHEN 'DEPOSIT'\n\
                    \t\t\tTHEN 'ADVANCE'\n\
                    \t\tWHEN 'DUE'\n\
                    \t\t\tTHEN 'ARREARS'\n\
                    \t\tELSE 'OTHER'\n\
                    \tEND AS payment_type\n\
                    ;";

    assert_expanded_layout(sql, expected);
}

#[test]
fn expanded_searched_case_aligns_when_and_else_after_a_leading_comma() {
    let sql = "SELECT id, CASE WHEN bank_id IS NOT NULL THEN 'REMOVED' \
               ELSE change_reason END AS reason, other_value FROM leases;";
    let expected = "SELECT\n\
                    \tid\n\
                    \t, CASE\n\
                    \t\t  WHEN bank_id IS NOT NULL\n\
                    \t\t\t  THEN 'REMOVED'\n\
                    \t\t  ELSE change_reason\n\
                    \t  END AS reason\n\
                    \t, other_value\n\
                    FROM\n\
                    \tleases\n\
                    ;";
    assert_expanded_layout(sql, expected);
}

#[test]
fn expanded_nested_case_keeps_each_branch_at_its_own_indent() {
    let sql = "SELECT CASE status WHEN 'A' THEN \
               CASE WHEN enabled THEN 'active' ELSE 'paused' END \
               ELSE 'closed' END;";
    let expected = "SELECT\n\
                    \tCASE status\n\
                    \t\tWHEN 'A'\n\
                    \t\t\tTHEN CASE\n\
                    \t\t\t\t     WHEN enabled\n\
                    \t\t\t\t\t     THEN 'active'\n\
                    \t\t\t\t     ELSE 'paused'\n\
                    \t\t\t     END\n\
                    \t\tELSE 'closed'\n\
                    \tEND\n\
                    ;";
    assert_expanded_layout(sql, expected);
}

#[test]
fn expanded_case_without_else_closes_at_the_case_indent() {
    let expected = "SELECT\n\
                    \tCASE\n\
                    \t\tWHEN enabled\n\
                    \t\t\tTHEN 'yes'\n\
                    \tEND\n\
                    ;";
    assert_expanded_layout("SELECT CASE WHEN enabled THEN 'yes' END;", expected);
}

#[test]
fn fit_case_stays_compact_when_it_fits() {
    for (sql, expected) in [
        (
            "SELECT CASE status WHEN 'A' THEN 'active' ELSE 'closed' END;",
            "select case status when 'A' then 'active' else 'closed' end;",
        ),
        (
            "SELECT CASE WHEN enabled THEN 'yes' END;",
            "select case when enabled then 'yes' end;",
        ),
    ] {
        assert_eq!(format_idempotently(sql, &FormatConfig::default()), expected);
    }
}

/// Column of the first non-blank character of the line that starts with `needle`.
fn column_of(output: &str, needle: &str) -> usize {
    let line = output
        .lines()
        .find(|line| line.trim_start().starts_with(needle))
        .unwrap_or_else(|| panic!("no line starting with {needle:?} in\n{output}"));
    line.len() - line.trim_start().len()
}

fn column_of_case(output: &str) -> usize {
    output
        .lines()
        .find_map(|line| line.find("case"))
        .expect("a line with case")
}

#[test]
fn expanded_case_aligns_from_where_it_starts_after_on() {
    let config = FormatConfig {
        layout: Layout::Expanded,
        clause_body_style: ClauseBodyStyle::Compact,
        indent_size: 4,
        ..Default::default()
    };
    let output = format_idempotently(
        "select * from t join u on case when t.a then t.b = u.b else t.c = u.c end",
        &config,
    );
    let case = column_of_case(&output);
    assert_eq!(column_of(&output, "when"), case + 4, "{output}");
    assert_eq!(column_of(&output, "then"), case + 8, "{output}");
    assert_eq!(column_of(&output, "else"), case + 4, "{output}");
    assert_eq!(column_of(&output, "end"), case, "{output}");
}

#[test]
fn expanded_case_aligns_from_where_it_starts_with_a_wide_indent() {
    for indent_style in [IndentStyle::Spaces, IndentStyle::Tabs] {
        let config = FormatConfig {
            layout: Layout::Expanded,
            clause_body_style: ClauseBodyStyle::Compact,
            indent_size: 8,
            indent_style: indent_style.clone(),
            ..Default::default()
        };
        let output = format_idempotently("select case when a then 1 else 2 end from t", &config);
        let expected = "select case\n               when a\n                       then 1\n               else 2\n       end\nfrom t;";
        let expected = match indent_style {
            IndentStyle::Spaces => expected.to_string(),
            // A tab per indent level that fits in the column, spaces for the rest.
            IndentStyle::Tabs => {
                "select case\n\t       when a\n\t\t       then 1\n\t       else 2\n       end\nfrom t;"
                    .to_string()
            }
        };
        assert_eq!(output, expected, "{indent_style:?}");
    }
}

#[test]
fn expanded_case_with_tabs_aligns_with_spaces_after_tabs() {
    let config = FormatConfig {
        layout: Layout::Expanded,
        indent_style: IndentStyle::Tabs,
        indent_size: 4,
        comma_style: CommaStyle::Leading,
        ..Default::default()
    };
    let output = format_idempotently("select a, case when b then 1 else 2 end from t", &config);
    assert_eq!(
        output,
        "select\n\ta\n\t, case\n\t\t  when b\n\t\t\t  then 1\n\t\t  else 2\n\t  end\nfrom\n\tt;"
    );
}

#[test]
fn when_condition_continuation_is_indented() {
    let config = FormatConfig {
        layout: Layout::Expanded,
        logical_operator_placement: LogicalOperatorPlacement::Leading,
        indent_size: 4,
        ..Default::default()
    };
    let output = format_idempotently(
        "select case when aaaa = 1 and bbbb = 2 then 'x' else 'y' end from t",
        &config,
    );
    assert_eq!(
        column_of(&output, "and"),
        column_of(&output, "when") + 4,
        "{output}"
    );

    // Fit layout indents the continuation the same way once the condition breaks.
    let config = FormatConfig {
        line_width: 40,
        ..Default::default()
    };
    let output = format_idempotently(
        "select case when first_condition_column = 1 and second_condition_column = 2 then 'x' else 'y' end from t",
        &config,
    );
    assert!(
        column_of(&output, "second_condition_column") > column_of(&output, "when"),
        "{output}"
    );
}

/// The example in `docs/features/formatting.md`.
#[test]
fn the_documented_expanded_case_layout() {
    let config = FormatConfig {
        layout: Layout::Expanded,
        comma_style: CommaStyle::Leading,
        logical_operator_placement: LogicalOperatorPlacement::Leading,
        ..Default::default()
    };
    assert_eq!(
        format_idempotently(
            "select id, case when status = 'active' and deleted_at is null then 'live' else 'archived' end as state from accounts",
            &config
        ),
        "select\n  id\n  , case\n      when status = 'active'\n        and deleted_at is null\n        then 'live'\n      else 'archived'\n    end as state\nfrom\n  accounts;"
    );
}
