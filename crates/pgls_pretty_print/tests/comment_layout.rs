use pgls_pretty_print::{
    FormatConfig, Layout, LogicalOperatorPlacement, format_statement,
    renderer::{IndentStyle, KeywordCase},
};

fn expanded_config() -> FormatConfig {
    FormatConfig {
        line_width: 200,
        indent_size: 4,
        indent_style: IndentStyle::Tabs,
        keyword_case: KeywordCase::Upper,
        layout: Layout::Expanded,
        logical_operator_placement: LogicalOperatorPlacement::Leading,
        isolate_semicolon: true,
        ..Default::default()
    }
}

fn format_idempotently(sql: &str, config: &FormatConfig) -> String {
    let ast = pgls_query::parse(sql).unwrap().into_root().unwrap();
    let first = format_statement(&ast, sql, config).unwrap().formatted;

    let ast = pgls_query::parse(&first).unwrap().into_root().unwrap();
    let second = format_statement(&ast, &first, config).unwrap().formatted;
    assert_eq!(second, first);
    first
}

fn assert_layout(sql: &str, expected: &str) {
    assert_eq!(format_idempotently(sql, &expanded_config()), expected);
}

#[test]
fn trailing_comment_keeps_the_simple_case_condition_together() {
    let output = format_idempotently(
        "SELECT CASE WHEN ecrictbimmsynd.codnatopr = 'SLD' -- automatic generated lines\n\
         THEN 'SLD' ELSE 'OTHER' END;",
        &expanded_config(),
    );
    assert!(
        output.contains("WHEN ecrictbimmsynd.codnatopr = 'SLD' -- automatic generated lines\n")
    );
    assert!(!output.contains("\n\n"));
}

#[test]
fn trailing_comment_does_not_hide_the_following_condition_in_fit_layout() {
    let output = format_idempotently(
        "SELECT * FROM t WHERE a = 1 -- first condition\nAND b = 2;",
        &FormatConfig::default(),
    );
    assert!(output.contains("a = 1 -- first condition\n"));
    assert!(output.contains("\n  and\n  b = 2;"), "{output}");
    assert!(!output.contains("\n\n"));
}

#[test]
fn standalone_comment_stays_before_a_parenthesized_operand() {
    let output = format_idempotently(
        "SELECT * FROM t WHERE a = 1\n-- grouped alternatives\nAND (b = 2 OR c = 3);",
        &expanded_config(),
    );
    assert!(output.contains("-- grouped alternatives\n\tAND ("));
}

#[test]
fn comment_after_the_conjunction_stays_after_the_conjunction() {
    let output = format_idempotently(
        "SELECT * FROM t WHERE a = 1 AND -- second condition\nb = 2;",
        &expanded_config(),
    );
    assert!(output.contains("AND -- second condition\n"), "{output}");
    assert!(output.contains("b = 2"));
    assert!(!output.contains("\n\n"));
}

#[test]
fn comment_after_an_opening_parenthesis_stays_with_the_next_operand() {
    for config in [FormatConfig::default(), expanded_config()] {
        let output = format_idempotently(
            "SELECT * FROM t WHERE (a >= '6000' AND a <= '6799')\n\
             OR ( -- non-numeric classes\ncoalesce(a, '') ~ '[A-Z]');",
            &config,
        )
        .to_ascii_lowercase();
        assert!(
            !output.contains("'6799' -- non-numeric classes"),
            "{output}"
        );
        let comment = output.find("-- non-numeric classes").unwrap();
        let operand = output.find("coalesce(").unwrap();
        assert!(comment < operand, "{output}");
        let between = &output[comment + "-- non-numeric classes".len()..operand];
        assert!(between.trim().is_empty(), "{output}");
    }
}

#[test]
fn trailing_comment_keeps_the_case_condition_together() {
    assert_layout(
        "SELECT CASE WHEN ecrictbimmsynd.codtyprept = 'B0'\n\
         AND planctb_tab.codplanctb < 700000 -- test\n\
         THEN 'CURRENT EXPENSES'\n\
         WHEN ecrictbimmsynd.codtyprept IS NOT NULL THEN 'REPAIR TO DO' END;",
        "SELECT\n\
         \tCASE WHEN ecrictbimmsynd.codtyprept = 'B0'\n\
         \t\tAND planctb_tab.codplanctb < 700000 -- test\n\
         \t\t\tTHEN 'CURRENT EXPENSES'\n\
         \t\tWHEN ecrictbimmsynd.codtyprept IS NOT NULL\n\
         \t\t\tTHEN 'REPAIR TO DO'\n\
         \tEND\n\
         ;",
    );
}

#[test]
fn standalone_comment_stays_before_the_conjunction_it_documents() {
    for keyword in ["AND", "OR"] {
        assert_layout(
            &format!(
                "SELECT CASE WHEN ecrictbimmsynd.codnatopr = 'SLD' THEN 'SLD'\n\
                 WHEN ecrictbimmsynd.codtyprept = 'B0'\n\
                 -- 700000 is a big number\n\
                 {keyword} planctb_tab.codplanctb < 700000 THEN 'CURRENT EXPENSES' END;"
            ),
            &format!(
                "SELECT\n\
                 \tCASE WHEN ecrictbimmsynd.codnatopr = 'SLD'\n\
                 \t\t\tTHEN 'SLD'\n\
                 \t\tWHEN ecrictbimmsynd.codtyprept = 'B0'\n\
                 \t\t-- 700000 is a big number\n\
                 \t\t{keyword} planctb_tab.codplanctb < 700000\n\
                 \t\t\tTHEN 'CURRENT EXPENSES'\n\
                 \tEND\n\
                 ;"
            ),
        );
    }
}
