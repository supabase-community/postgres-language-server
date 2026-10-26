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
