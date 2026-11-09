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

fn assert_comment_tokens(sql: &str, expected: &[&str]) {
    for output in format_both_layouts(sql) {
        let scan = pgls_query::scan(&output).unwrap();
        let tokens: Vec<_> = scan
            .tokens
            .iter()
            .map(|token| output[token.start as usize..token.end as usize].trim_end())
            .collect();
        assert!(
            tokens
                .windows(expected.len())
                .any(|window| window == expected),
            "expected consecutive tokens {expected:?} in:\n{output}"
        );
    }
}

#[test]
fn consecutive_comments_stay_in_order_before_case_keywords() {
    for (comments, first, second) in [
        ("/* first */ /* second */", "/* first */", "/* second */"),
        ("/* first */ -- second\n", "/* first */", "-- second"),
        ("-- first\n/* second */", "-- first", "/* second */"),
        ("\n-- first\n-- second\n", "-- first", "-- second"),
    ] {
        assert_comment_tokens(
            &format!("select case when a then 1 {comments} else 2 end from t"),
            &["1", first, second, "else", "2"],
        );
        assert_comment_tokens(
            &format!("select case when a {comments} then 1 else 2 end from t"),
            &["a", first, second, "then", "1"],
        );
        assert_comment_tokens(
            &format!("select case when a then 1 else 2 {comments} end from t"),
            &["2", first, second, "end"],
        );
    }
}

#[test]
fn consecutive_comments_stay_in_order_before_grant_and_revoke_keywords() {
    for (comments, first, second) in [
        ("/* first */ /* second */", "/* first */", "/* second */"),
        ("/* first */ -- second\n", "/* first */", "-- second"),
        ("-- first\n/* second */", "-- first", "/* second */"),
        ("\n-- first\n-- second\n", "-- first", "-- second"),
    ] {
        for (sql, previous, keyword) in [
            (format!("grant select {comments} on t to u"), "select", "on"),
            (format!("grant select on t {comments} to u"), "t", "to"),
            (
                format!("revoke select {comments} on t from u"),
                "select",
                "on",
            ),
            (format!("revoke select on t {comments} from u"), "t", "from"),
            (format!("grant r1 {comments} to u"), "r1", "to"),
            (format!("revoke r1 {comments} from u"), "r1", "from"),
        ] {
            assert_comment_tokens(&sql, &[previous, first, second, keyword]);
        }
    }
}

#[test]
fn comments_before_end_follow_the_complete_branch_expression() {
    for (expression, last_token) in [
        ("f(2)", ")"),
        ("array[1, 2]", "]"),
        ("2::integer", ")"),
        ("(select 2)", ")"),
    ] {
        for comments in ["/* last */", "\n-- last\n"] {
            let comment = comments.trim();
            for branch in ["then", "else"] {
                let sql = if branch == "then" {
                    format!("select case when a then {expression} {comments} end from t")
                } else {
                    format!("select case when a then 1 else {expression} {comments} end from t")
                };
                assert_comment_tokens(&sql, &[last_token, comment, "end"]);
            }
        }
    }
}

#[test]
fn comments_before_end_stay_with_the_case_they_close() {
    for branch in ["then", "else"] {
        let prefix = if branch == "then" {
            "select case when a then"
        } else {
            "select case when a then 1 else"
        };
        for (inner, outer) in [
            ("/* inner */", "/* outer */"),
            ("\n-- inner\n", "\n-- outer\n"),
        ] {
            let sql = format!(
                "{prefix} case when b then 'case end' else 3 {inner} end {outer} end from t"
            );
            assert_comment_tokens(&sql, &["3", inner.trim(), "end", outer.trim(), "end"]);
        }
    }
}

#[test]
fn consecutive_comments_before_end_stay_after_the_inner_case() {
    assert_comment_tokens(
        "select case when a then 1 else case when b then 2 else 3 end \
         /* first */ /* second */ end from t",
        &["end", "/* first */", "/* second */", "end"],
    );
}

#[test]
fn keyword_names_do_not_change_case_end_ownership() {
    for (expression, last_token) in [
        ("t.end", "\"end\""),
        ("t.case", "\"case\""),
        ("(select 2 end)", ")"),
        ("(select 2 as end)", ")"),
        ("(select 2 case)", ")"),
        ("(select 2 as case)", ")"),
    ] {
        assert_comment_tokens(
            &format!("select case when a then {expression} /* last */ end from t"),
            &[last_token, "/* last */", "end"],
        );
    }
}
