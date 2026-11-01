use pgls_pretty_print::{
    CommaStyle, FormatConfig, IndentStyle, KeywordCase, Layout, LogicalOperatorPlacement,
    format_statement,
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
         \tCASE\n\
         \t\tWHEN ecrictbimmsynd.codtyprept = 'B0'\n\
         \t\t\tAND planctb_tab.codplanctb < 700000 -- test\n\
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
                 \tCASE\n\
                 \t\tWHEN ecrictbimmsynd.codnatopr = 'SLD'\n\
                 \t\t\tTHEN 'SLD'\n\
                 \t\tWHEN ecrictbimmsynd.codtyprept = 'B0'\n\
                 \t\t\t-- 700000 is a big number\n\
                 \t\t\t{keyword} planctb_tab.codplanctb < 700000\n\
                 \t\t\tTHEN 'CURRENT EXPENSES'\n\
                 \tEND\n\
                 ;"
            ),
        );
    }
}

fn format_both_layouts(sql: &str) -> [String; 2] {
    let expanded = FormatConfig {
        layout: Layout::Expanded,
        ..Default::default()
    };
    [
        format_idempotently(sql, &FormatConfig::default()),
        format_idempotently(sql, &expanded),
    ]
}

#[test]
fn a_comment_before_else_stays_before_else() {
    for output in
        format_both_layouts("select case when a then 'x' /* inline */ else 'y' end from t")
    {
        assert!(output.contains("'x' /* inline */"), "{output}");
    }
    for output in
        format_both_layouts("select case when a then 'x'\n-- before else\nelse 'y' end from t")
    {
        assert!(output.contains("-- before else\n"), "{output}");
        let comment = output.find("-- before else").unwrap();
        assert!(comment < output.find("else 'y'").unwrap(), "{output}");
    }
}

#[test]
fn a_comment_before_then_stays_before_then() {
    for output in format_both_layouts("select case when a /* cond */ then 1 else 2 end from t") {
        assert!(output.contains("a /* cond */"), "{output}");
        assert!(!output.contains("then /* cond */"), "{output}");
    }
}

#[test]
fn a_comment_before_end_stays_inside_the_case() {
    for output in format_both_layouts("select case when a then 1 else 2 /* last */ end from t") {
        assert!(output.contains("2 /* last */"), "{output}");
        assert!(!output.contains("/* last */ t"), "{output}");
    }
}

#[test]
fn a_comment_after_case_stays_before_the_first_when() {
    for output in format_both_layouts("select case -- why\nwhen a then 'x' else 'y' end from t") {
        let comment = output.find("-- why").unwrap();
        assert!(comment < output.find("when a").unwrap(), "{output}");
        assert!(!output.contains("'y' -- why"), "{output}");
    }
}

#[test]
fn a_comment_before_on_in_a_grant_stays_before_on() {
    assert_eq!(
        format_idempotently("grant select -- c\non t to u", &FormatConfig::default()),
        "grant select -- c\n  on table t\n  to u;"
    );
    assert_eq!(
        format_idempotently("grant select on t /* c */ to u", &FormatConfig::default()),
        "grant select on table t /* c */ to u;"
    );
    assert_eq!(
        format_idempotently("grant r1 /* c */ to u", &FormatConfig::default()),
        "grant r1 /* c */ to u;"
    );
}

fn leading_comma_config() -> FormatConfig {
    FormatConfig {
        comma_style: CommaStyle::Leading,
        ..Default::default()
    }
}

#[test]
fn a_fill_list_with_leading_commas_keeps_an_own_line_comment_before_the_comma() {
    for sql in [
        "select * from t where x in (1, 2\n-- c\n, 3)",
        "select * from t where x in (1, 2,\n-- c\n3)",
    ] {
        assert_eq!(
            format_idempotently(sql, &leading_comma_config()),
            "select\n  *\nfrom\n  t\nwhere\n  x\n  in (\n    1, 2\n    -- c\n    , 3\n  );"
        );
    }
    // The comment leads `y`, the first node of the item, not the operator the item's location
    // points to.
    assert_eq!(
        format_idempotently(
            "select * from t where x in (1, 2\n-- c\n, y + 3)",
            &leading_comma_config()
        ),
        "select\n  *\nfrom\n  t\nwhere\n  x\n  in (\n    1, 2\n    -- c\n    , y + 3\n  );"
    );
    assert_eq!(
        format_idempotently("select array[1, 2\n-- c\n, 3]", &leading_comma_config()),
        "select\n  array[\n    1, 2\n    -- c\n    , 3\n  ];"
    );
    assert_eq!(
        format_idempotently(
            "grant select on t1, t2\n-- c\n, t3 to u",
            &leading_comma_config()
        ),
        "grant select\n  on table t1, t2\n    -- c\n    , t3\n  to u;"
    );
}
