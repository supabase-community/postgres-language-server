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

#[test]
fn a_comment_before_a_comma_stays_before_the_comma() {
    for config in [FormatConfig::default(), leading_comma_config()] {
        assert_eq!(
            format_idempotently("select f(a) /* c */, b + 1 /* d */, c from t", &config),
            "select f(a) /* c */, b + 1 /* d */, c from t;"
        );
        assert_eq!(
            format_idempotently("select * from t where x in (1 /* one */, 2)", &config),
            "select * from t where x in (1 /* one */, 2);"
        );
    }
    assert_eq!(
        format_idempotently("select a /* c */, b from t", &expanded_config()),
        "SELECT\n\ta /* c */,\n\tb\nFROM\n\tt\n;"
    );
}

#[test]
fn a_comment_before_a_clause_keyword_stays_before_the_keyword() {
    for (sql, expected) in [
        ("select a /* c */ from t", "select a /* c */ from t;"),
        (
            "select a from t /* c */ where x = 1",
            "select a from t /* c */ where x = 1;",
        ),
        (
            "select a from t\n-- c\nwhere x = 1",
            "select\n  a\nfrom\n  t\n-- c\nwhere\n  x = 1;",
        ),
        (
            "select a from t where x = 1 /* c */ group by a /* d */ having count(*) > 1 /* e */ \
             order by a /* f */ limit 1 /* g */ offset 2",
            "select\n  a\nfrom\n  t\nwhere\n  x = 1 /* c */\ngroup by a /* d */\nhaving\n  \
             count(*) > 1 /* e */\norder by a /* f */\nlimit 1 /* g */\noffset 2;",
        ),
        (
            "select a from t order by a /* c */ fetch first 3 rows only",
            "select a from t order by a /* c */ limit 3;",
        ),
        (
            "select 1 /* c */ union select 2",
            "select 1 /* c */ union select 2;",
        ),
        (
            "update t /* c */ set a = 1 /* d */ from u /* e */ where t.id = u.id /* f */ returning a",
            "update t /* c */ set a = 1 /* d */ from u /* e */ where t.id = u.id /* f */ returning a;",
        ),
        (
            "delete from t /* c */ using u /* d */ where t.id = u.id",
            "delete from t /* c */ using u /* d */ where t.id = u.id;",
        ),
        (
            "merge into t /* c */ using u on t.id = u.id when matched then do nothing",
            "merge into t /* c */ using u on t.id = u.id when matched then do nothing;",
        ),
    ] {
        assert_eq!(format_idempotently(sql, &FormatConfig::default()), expected);
    }
}

#[test]
fn a_comment_before_a_join_keyword_stays_before_the_keyword() {
    for (sql, expected) in [
        (
            "select a from t /* c */ join u on t.id = u.id",
            "select a from t /* c */ inner join u on t.id = u.id;",
        ),
        (
            "select a from t /* c */ left join u on t.id = u.id",
            "select a from t /* c */ left outer join u on t.id = u.id;",
        ),
        (
            "select a from t join u /* c */ on t.id = u.id",
            "select a from t inner join u /* c */ on t.id = u.id;",
        ),
    ] {
        assert_eq!(format_idempotently(sql, &FormatConfig::default()), expected);
    }
}

#[test]
fn a_comment_before_as_stays_before_the_alias() {
    for (sql, expected) in [
        (
            "select a /* c */ as x, b from t",
            "select a /* c */ as x, b from t;",
        ),
        (
            "select f(a) /* c */ as x, b from t",
            "select f(a) /* c */ as x, b from t;",
        ),
        (
            "select a from t /* c */ as x where true",
            "select a from t /* c */ as x where true;",
        ),
    ] {
        assert_eq!(format_idempotently(sql, &FormatConfig::default()), expected);
    }
}

#[test]
fn a_comment_before_a_comma_stays_before_the_comma_in_every_list() {
    for config in [FormatConfig::default(), leading_comma_config()] {
        for sql in [
            "update t set a = 1 /* c */, b = 2;",
            "select sum(x) over w1 from t window w1 as (order by a) /* c */, w2 as (order by b);",
            "select f(a, b /* c */, variadic c) from t;",
            "select distinct on (a /* c */, b) a from t;",
            "alter foreign table t alter column a options (add x 'y' /* c */, add z 'w');",
        ] {
            let output = format_idempotently(sql, &config);
            assert!(output.contains("/* c */,"), "{output}");
        }
    }
    assert_eq!(
        format_idempotently(
            "create function f(a int /* c */, b int) returns int language sql as 'select 1'",
            &FormatConfig::default()
        ),
        "create function f(a int /* c */, b int)\nreturns int\nlanguage sql\nas $$select 1$$;"
    );
    assert_eq!(
        format_idempotently(
            "alter table t add column a int /* c */, add column b int",
            &FormatConfig::default()
        ),
        "alter table t\n  add column a int /* c */,\n  add column b int;"
    );
}

#[test]
fn lists_that_wrote_their_own_commas_follow_the_comma_style() {
    assert_eq!(
        format_idempotently(
            "create table t (a int,\n-- c\nb int)",
            &leading_comma_config()
        ),
        "create table t (\n  a int\n  -- c\n  , b int\n);"
    );
    assert_eq!(
        format_idempotently(
            "alter table t add column a int, add column b int",
            &leading_comma_config()
        ),
        "alter table t\n  add column a int\n  , add column b int;"
    );
}

#[test]
fn a_comment_before_a_dml_keyword_stays_before_the_keyword() {
    for (sql, expected) in [
        (
            "select a /* c */ into t2 from t",
            "select a /* c */ into t2 from t;",
        ),
        (
            "insert into t values (1) /* c */ on conflict do nothing",
            "insert into t values (1) /* c */ on conflict do nothing;",
        ),
        (
            "insert into t values (1) on conflict (a) /* c */ do update set b = 1 /* d */ where t.b > 0",
            "insert into t values (1) on conflict (a) /* c */ do update set b = 1 /* d */ where t.b > 0;",
        ),
        (
            "insert into t values (1) on conflict (a) /* c */ where a > 0 do nothing",
            "insert into t values (1) on conflict (a) /* c */ where a > 0 do nothing;",
        ),
        (
            "merge into t using u on t.id = u.id when matched then delete /* d */ \
             when not matched then insert values (1)",
            "merge into t\nusing u\non t.id = u.id\nwhen matched then delete /* d */\n\
             when not matched by target then insert values (1);",
        ),
        (
            "merge into t using u on t.id = u.id when matched and t.a > 0 /* c */ \
             then update set a = 1",
            "merge into t using u on t.id = u.id when matched and t.a > 0 /* c */ then update set a = 1;",
        ),
    ] {
        assert_eq!(format_idempotently(sql, &FormatConfig::default()), expected);
    }
}

#[test]
fn a_comment_before_a_ddl_keyword_stays_before_the_keyword() {
    for (sql, expected) in [
        (
            "create index i on t /* c */ using btree (a) /* d */ where a > 0",
            "create index i on t /* c */ using btree (a) /* d */ where a > 0;",
        ),
        (
            "create view v /* c */ as select 1",
            "create view v /* c */\nas select 1;",
        ),
        (
            "create table t2 /* c */ as select 1",
            "create table t2 /* c */ as select 1;",
        ),
        (
            "create table t (a int) /* c */ partition by range (a)",
            "create table t (a int) /* c */\npartition by range(a);",
        ),
        (
            "create policy p on t /* c */ for select to u /* d */ using (true) /* e */ \
             with check (true)",
            "create policy p\non t\nas permissive /* c */\nfor select\nto u /* d */\n\
             using (true) /* e */\nwith check (true);",
        ),
    ] {
        assert_eq!(format_idempotently(sql, &FormatConfig::default()), expected);
    }
}
