use pgls_pretty_print::{
    ClauseBodyStyle, FormatConfig, Layout, LogicalOperatorPlacement, format_statement,
};

fn format_idempotently(sql: &str, config: &FormatConfig) -> String {
    let ast = pgls_query::parse(sql).unwrap().into_root().unwrap();
    let first = format_statement(&ast, sql, config).unwrap().formatted;
    let ast = pgls_query::parse(&first).unwrap().into_root().unwrap();
    let second = format_statement(&ast, &first, config).unwrap().formatted;
    assert_eq!(first, second);
    first
}

fn expanded_compact_leading() -> FormatConfig {
    FormatConfig {
        layout: Layout::Expanded,
        clause_body_style: ClauseBodyStyle::Compact,
        logical_operator_placement: LogicalOperatorPlacement::Leading,
        ..Default::default()
    }
}

#[test]
fn scalar_subquery_keeps_operator_and_parenthesis_together() {
    assert_eq!(
        format_idempotently(
            "create policy p on t using (org_id = (select org_id from members where user_id = auth.uid()))",
            &expanded_compact_leading(),
        ),
        "create policy p\non t\nas permissive\nfor all\nto public\nusing (\n  org_id = (\n    select org_id\n    from members\n    where user_id = auth.uid()\n  )\n);"
    );

    let config = FormatConfig {
        line_width: 60,
        ..Default::default()
    };
    assert_eq!(
        format_idempotently(
            "select * from t where t.owner_id = (select m.user_id from memberships as m where m.id = 1)",
            &config,
        ),
        "select\n  *\nfrom\n  t\nwhere\n  t.owner_id = (\n    select m.user_id from memberships as m where m.id = 1\n  );"
    );
}

#[test]
fn array_subquery_keeps_operator_and_array_together() {
    let config = FormatConfig {
        line_width: 60,
        ..Default::default()
    };
    let output = format_idempotently(
        "select * from t where t.tags = array(select tag from tags where tags.id = t.id order by tag)",
        &config,
    );
    assert!(output.contains("t.tags = array(\n"), "{output}");
}

#[test]
fn nested_subquery_inside_select_list_only_subquery_expands() {
    let config = FormatConfig {
        layout: Layout::Expanded,
        clause_body_style: ClauseBodyStyle::Compact,
        ..Default::default()
    };
    let output = format_idempotently(
        "select * from t where a = (select coalesce((select max(x) from y where y.id = t.id), 0))",
        &config,
    );
    assert!(
        output
            .lines()
            .any(|line| line.trim_start().starts_with("from y")),
        "{output}"
    );
}

#[test]
fn a_subquery_opened_on_the_where_line_closes_under_where() {
    let config = FormatConfig {
        layout: Layout::Expanded,
        clause_body_style: ClauseBodyStyle::Compact,
        ..Default::default()
    };
    assert_eq!(
        format_idempotently(
            "select * from t where exists (select 1 from u where u.id = t.id)",
            &config,
        ),
        "select *\nfrom t\nwhere exists (\n  select 1\n  from u\n  where u.id = t.id\n);"
    );
    assert_eq!(
        format_idempotently(
            "select * from t where t.a = (select max(x) from u where u.id = t.id)",
            &config,
        ),
        "select *\nfrom t\nwhere t.a = (\n  select max(x)\n  from u\n  where u.id = t.id\n);"
    );

    // A chain still indents its continuation lines, and a subquery in it closes on its own line.
    let output = format_idempotently(
        "select * from t where t.a = 1 and exists (select 1 from u where u.id = t.id)",
        &config,
    );
    assert!(output.contains("\n  exists (\n    select 1\n"), "{output}");
    assert!(output.ends_with("\n  );"), "{output}");
}

#[test]
fn a_subquery_opened_on_the_on_line_closes_under_on() {
    let config = FormatConfig {
        line_width: 60,
        ..Default::default()
    };
    let output = format_idempotently(
        "select * from t inner join u on u.id = (select max(id) from versions where versions.name = t.name)",
        &config,
    );
    assert!(output.contains("\n  on u.id = (\n    select\n"), "{output}");
    assert!(output.ends_with("\n  );"), "{output}");
}

#[test]
fn a_lone_compact_select_item_closes_under_select() {
    let config = FormatConfig {
        clause_body_style: ClauseBodyStyle::Compact,
        line_width: 60,
        ..Default::default()
    };
    assert_eq!(
        format_idempotently(
            "select coalesce(some_long_column_name, another_long_column_name, 0) from t",
            &config,
        ),
        "select coalesce(\n  some_long_column_name,\n  another_long_column_name,\n  0\n)\nfrom t;"
    );

    // An item with continuation lines of its own keeps the indent for them.
    let output = format_idempotently(
        "select some_long_column_name + another_long_column_name + a_third_long_column from t",
        &config,
    );
    assert_eq!(
        output,
        "select some_long_column_name + another_long_column_name +\n  a_third_long_column\nfrom t;"
    );
}
