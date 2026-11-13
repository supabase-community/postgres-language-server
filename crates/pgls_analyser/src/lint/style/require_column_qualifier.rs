use std::collections::HashSet;

use crate::utils::statement_nodes;
use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::declare_lint_rule;
use pgls_catalog::resolve::{Reference, figure_column_name};
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::protobuf::{
    CoercionForm, ColumnRef, Node, ReturningClause, SelectStmt, SetOperation,
};
use pgls_query::{NodeEnum, NodeRef};
use pgls_query_ext::utils::string_value;

declare_lint_rule! {
    /// Columns in queries should be qualified with their table.
    ///
    /// Without a table name or alias, Postgres looks the column up in every relation of the
    /// query, and then in the ones of the queries around it. If another relation gets a column
    /// with the same name, the query fails, or a subquery silently refers to the column of the
    /// outer query. In the body of a SQL function, a column with the name of a parameter takes
    /// its place.
    ///
    /// The rule is meant for code where this matters, like functions with `SECURITY DEFINER` and
    /// extension scripts, as defence in depth next to qualifying relations, functions, types and
    /// operators with their schema.
    ///
    /// The rule checks the column references of queries and DML that read from or write to a
    /// relation. It does not report:
    ///
    /// - queries without relations, like `select 1`
    /// - references to output columns or their positions in `ORDER BY` and `GROUP BY`, and the
    ///   `ORDER BY` of `UNION`, `INTERSECT` and `EXCEPT`
    /// - whole-row references, like `select u from public.users u`
    /// - the parameters of the SQL function the statement is in
    ///
    /// A name of an output column, relation, alias or parameter is still reported where Postgres
    /// resolves it to a column, like `u` in `select u from public.t u` if `t` has a column `u`,
    /// or `n` in `select u.id as n from public.t u group by n` if `t` has a column `n`: in
    /// `GROUP BY`, unlike in `ORDER BY`, the columns of `FROM` take precedence over the output
    /// columns. This needs the columns of the relations from the database.
    ///
    /// - the column lists of `INSERT`, `USING (...)` and `ON CONFLICT (...)`, and the targets of
    ///   `UPDATE ... SET`, which can only name columns of one table
    /// - expressions outside queries, like column definitions, defaults, `CHECK` constraints and
    ///   index expressions
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// select relid from pg_catalog.pg_statio_all_tables;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// select at.relid from pg_catalog.pg_statio_all_tables as at;
    /// select at.relid as id from pg_catalog.pg_statio_all_tables as at order by id;
    /// insert into public.users (id, name) values (1, 'alice') on conflict (id) do nothing;
    /// ```
    ///
    pub RequireColumnQualifier {
        version: "next",
        name: "requireColumnQualifier",
        severity: Severity::Warning,
        recommended: false,
    }
}

impl LinterRule for RequireColumnQualifier {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let tokens = ctx.tokens();
        // The column references of the queries that have relations, by address. A reference in
        // a subquery without relations can still refer to a column of the query around it.
        let mut in_query: HashSet<*const ColumnRef> = HashSet::new();
        // The references that are not reported, see the rule documentation.
        let mut skipped: HashSet<*const ColumnRef> = HashSet::new();
        // The `GROUP BY` items that are names of output columns, by address. A column of `FROM`
        // with the name takes precedence.
        let mut grouped_outputs: HashSet<*const ColumnRef> = HashSet::new();
        // The names of relations and aliases, which a single name can refer to as a whole row.
        let mut relation_names: HashSet<&str> = HashSet::new();
        for node in statement_nodes(ctx.stmt()) {
            match node {
                NodeRef::SelectStmt(select) => {
                    if !select.from_clause.is_empty() {
                        in_query.extend(column_refs(node));
                    }
                    let (sorted, grouped) = output_column_references(select);
                    skipped.extend(sorted);
                    grouped_outputs.extend(grouped);
                }
                NodeRef::UpdateStmt(_) | NodeRef::DeleteStmt(_) | NodeRef::MergeStmt(_) => {
                    in_query.extend(column_refs(node));
                }
                // The query of `INSERT ... SELECT` is a `SelectStmt` of its own, and `VALUES`
                // can't reference columns.
                NodeRef::InsertStmt(insert) => {
                    let on_conflict = insert.on_conflict_clause.iter().map(|on| on.to_ref());
                    let returning = insert.returning_clause.iter().map(ReturningClause::to_ref);
                    for part in on_conflict.chain(returning) {
                        in_query.extend(column_refs(part));
                    }
                }
                NodeRef::InferClause(_) => skipped.extend(column_refs(node)),
                NodeRef::RangeVar(relation) => {
                    relation_names.insert(&relation.relname);
                }
                NodeRef::Alias(alias) => {
                    relation_names.insert(&alias.aliasname);
                }
                NodeRef::CommonTableExpr(cte) => {
                    relation_names.insert(&cte.ctename);
                }
                NodeRef::ReturningClause(_) => {
                    relation_names.extend(["old", "new"]);
                }
                NodeRef::ReturningOption(option) => {
                    relation_names.insert(&option.value);
                }
                _ => {}
            }
        }
        let parameters: HashSet<&str> = ctx
            .function()
            .into_iter()
            .flat_map(|function| &function.params)
            .filter_map(|param| param.name.as_deref())
            .collect();

        // The references, and whether they may refer to something else than a column: a whole
        // row, a parameter, or an output column.
        let mut columns: Vec<(&str, i32, bool)> = statement_nodes(ctx.stmt())
            .into_iter()
            .filter_map(|node| match node {
                NodeRef::ColumnRef(column)
                    if in_query.contains(&std::ptr::from_ref(column))
                        && !skipped.contains(&std::ptr::from_ref(column)) =>
                {
                    let name = single_name(column)?;
                    let maybe_other = relation_names.contains(name)
                        || parameters.contains(name)
                        || grouped_outputs.contains(&std::ptr::from_ref(column));
                    Some((name, column.location, maybe_other))
                }
                _ => None,
            })
            .collect();
        // Such a reference is reported only if Postgres resolves it to a column, which only the
        // resolver knows, with the columns of the relations in scope.
        if columns.iter().any(|(_, _, maybe_other)| *maybe_other) {
            let resolved: HashSet<i32> = ctx
                .references()
                .into_iter()
                .flatten()
                .filter_map(|reference| match reference {
                    Reference::Column { location, .. } => Some(*location),
                    _ => None,
                })
                .collect();
            columns
                .retain(|(_, location, maybe_other)| !maybe_other || resolved.contains(location));
        }
        columns.sort_by_key(|(_, location, _)| *location);
        columns
            .into_iter()
            .map(|(name, location, _)| {
                LinterDiagnostic::new(
                    rule_category!(),
                    tokens.and_then(|tokens| tokens.range_at(location)),
                    markup! {
                        "Column "<Emphasis>{name}</Emphasis>" is not qualified with its table."
                    },
                )
                .detail(
                    None,
                    "Without a table, the column is looked up in every relation of the query and of the queries around it, so a column with the same name in another relation can take its place.",
                )
                .note(markup! {
                    "Qualify the column with the name or alias of its table, e.g. "<Emphasis>"t."{name}</Emphasis>"."
                })
            })
            .collect()
    }
}

/// The column references in `node`, including `node` itself, by address.
fn column_refs(node: NodeRef) -> impl Iterator<Item = *const ColumnRef> {
    node.iter().filter_map(|node| match node {
        NodeRef::ColumnRef(column) => Some(std::ptr::from_ref(column)),
        _ => None,
    })
}

/// The name of a column reference that is a single name, like `id`, but not `*` or `u.id`.
fn single_name(column: &ColumnRef) -> Option<&str> {
    match column.fields.as_slice() {
        [name] => string_value(name),
        _ => None,
    }
}

/// The `ORDER BY` and `GROUP BY` items of `select` that are single names of its output columns,
/// by address, the ones of `ORDER BY` first. In the `ORDER BY` of `UNION`, `INTERSECT` and
/// `EXCEPT`, every single name is an output column.
fn output_column_references(select: &SelectStmt) -> (Vec<*const ColumnRef>, Vec<*const ColumnRef>) {
    let is_set_operation = !matches!(
        select.op(),
        SetOperation::SetopNone | SetOperation::Undefined
    );
    let output_names: HashSet<String> = select
        .target_list
        .iter()
        .filter_map(|target| match &target.node {
            Some(NodeEnum::ResTarget(target)) if !target.name.is_empty() => {
                Some(target.name.clone())
            }
            Some(NodeEnum::ResTarget(target)) => {
                figure_column_name(target.val.as_ref()?.node.as_ref()?)
            }
            _ => None,
        })
        .collect();
    let output_reference = |item: &Node, is_output: bool| match &item.node {
        Some(NodeEnum::ColumnRef(column))
            if is_output || single_name(column).is_some_and(|name| output_names.contains(name)) =>
        {
            Some(std::ptr::from_ref(column))
        }
        _ => None,
    };
    let sorted = select
        .sort_clause
        .iter()
        .filter_map(|item| match &item.node {
            Some(NodeEnum::SortBy(sort_by)) => sort_by.node.as_deref(),
            _ => None,
        })
        .filter_map(|item| output_reference(item, is_set_operation))
        .collect();
    let grouped = group_by_items(&select.group_clause)
        .into_iter()
        .filter_map(|item| output_reference(item, false))
        .collect();
    (sorted, grouped)
}

/// The items of `GROUP BY`, with the ones of `GROUPING SETS`, `ROLLUP` and `CUBE`, and of the
/// parenthesized lists in them, instead of the grouping sets and lists. Like
/// [`flatten_grouping_sets`], before Postgres looks up their names. Every node of the clause is
/// visited at most once, without recursion.
///
/// [`flatten_grouping_sets`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_clause.c#L2258
fn group_by_items(group_clause: &[Node]) -> Vec<&Node> {
    let mut pending: Vec<&Node> = group_clause.iter().collect();
    let mut items = Vec::new();
    while let Some(node) = pending.pop() {
        match &node.node {
            Some(NodeEnum::GroupingSet(set)) => pending.extend(&set.content),
            Some(NodeEnum::RowExpr(row))
                if row.row_format() == CoercionForm::CoerceImplicitCast =>
            {
                pending.extend(&row.args);
            }
            _ => items.push(node),
        }
    }
    items
}

#[cfg(test)]
mod tests {
    fn spans(sql: &str) -> Vec<String> {
        crate::spanned_texts("style", "requireColumnQualifier", sql)
    }

    #[test]
    fn points_at_the_column() {
        assert_eq!(
            spans("select \"Id\", u.name, * from public.users u where email = $1"),
            ["\"Id\"", "email"]
        );
        assert_eq!(
            spans("update public.users set name = upper(name) returning id"),
            ["name", "id"]
        );
    }

    #[test]
    fn reports_references_to_outer_queries() {
        assert_eq!(spans("select (select id) from public.users u"), ["id"]);
    }

    #[test]
    fn skips_output_columns() {
        assert_eq!(
            spans(
                "select u.id as n, u.name from public.users u group by n, name order by n, name, 1, lower(email)"
            ),
            ["email"]
        );
        assert!(
            spans(
                "select u.id from public.users u union select p.id from public.posts p order by id"
            )
            .is_empty()
        );
    }

    #[test]
    fn skips_output_columns_through_casts_and_grouping_sets() {
        assert!(
            spans(
                "select u.name::pg_catalog.text, u.id as n from public.users u \
                 group by grouping sets ((n), rollup (n, name), cube ((n, name))), (n, name) \
                 order by name"
            )
            .is_empty()
        );
        assert_eq!(
            spans("select u.id as n from public.users u group by grouping sets ((n, email))"),
            ["email"]
        );
    }

    #[test]
    fn skips_whole_rows_and_queries_without_relations() {
        assert!(spans("select u from public.users u").is_empty());
        assert!(spans("select 1, now()").is_empty());
    }

    #[test]
    fn skips_insert_columns_and_conflict_targets() {
        assert_eq!(
            spans(
                "insert into public.users (id, name) values (1, 'a') \
                 on conflict (id) where active do update set name = excluded.name \
                 returning id"
            ),
            ["id"]
        );
    }

    #[test]
    fn skips_ddl() {
        assert!(spans("create table public.t (a int check (a > 0), b int default 1)").is_empty());
        assert!(spans("create index on public.t (lower(a)) where b > 0").is_empty());
    }
}
