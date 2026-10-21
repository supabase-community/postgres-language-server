use pgls_pretty_print::{
    CommaStyle, FormatConfig, Layout, LogicalOperatorPlacement, format_statement,
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
