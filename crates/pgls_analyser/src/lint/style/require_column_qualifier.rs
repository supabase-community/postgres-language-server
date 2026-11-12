use std::collections::HashSet;

use crate::utils::statement_nodes;
use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::declare_lint_rule;
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::protobuf::{ColumnRef, Node, ReturningClause, SelectStmt, SetOperation};
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
        // The names of relations and aliases, which a single name can refer to as a whole row.
        let mut relation_names: HashSet<&str> = HashSet::new();
        for node in statement_nodes(ctx.stmt()) {
            match node {
                NodeRef::SelectStmt(select) => {
                    if !select.from_clause.is_empty() {
                        in_query.extend(column_refs(node));
                    }
                    skipped.extend(output_column_references(select));
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

        let mut columns: Vec<(&str, i32)> = statement_nodes(ctx.stmt())
            .into_iter()
            .filter_map(|node| match node {
                NodeRef::ColumnRef(column)
                    if in_query.contains(&std::ptr::from_ref(column))
                        && !skipped.contains(&std::ptr::from_ref(column)) =>
                {
                    Some((single_name(column)?, column.location))
                }
                _ => None,
            })
            .filter(|(name, _)| !relation_names.contains(name) && !parameters.contains(name))
            .collect();
        columns.sort_by_key(|(_, location)| *location);
        columns
            .into_iter()
            .map(|(name, location)| {
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
/// by address. In the `ORDER BY` of `UNION`, `INTERSECT` and `EXCEPT`, every single name is an
/// output column.
fn output_column_references(select: &SelectStmt) -> Vec<*const ColumnRef> {
    let is_set_operation = !matches!(
        select.op(),
        SetOperation::SetopNone | SetOperation::Undefined
    );
    let output_names: HashSet<&str> = select
        .target_list
        .iter()
        .filter_map(|target| match &target.node {
            Some(NodeEnum::ResTarget(target)) => output_name(target),
            _ => None,
        })
        .collect();
    let sorted = select
        .sort_clause
        .iter()
        .filter_map(|item| match &item.node {
            Some(NodeEnum::SortBy(sort_by)) => sort_by.node.as_deref(),
            _ => None,
        });
    sorted
        .map(|item| (item, is_set_operation))
        .chain(select.group_clause.iter().map(|item| (item, false)))
        .filter_map(|(item, is_output)| match &item.node {
            Some(NodeEnum::ColumnRef(column))
                if is_output
                    || single_name(column).is_some_and(|name| output_names.contains(name)) =>
            {
                Some(std::ptr::from_ref(column))
            }
            _ => None,
        })
        .collect()
}

/// The name of an output column, like Postgres names it for the simple cases: its alias, or the
/// name of the column or function it is.
fn output_name(target: &pgls_query::protobuf::ResTarget) -> Option<&str> {
    if !target.name.is_empty() {
        return Some(&target.name);
    }
    fn last(names: &[Node]) -> Option<&str> {
        names.last().and_then(string_value)
    }
    match target.val.as_ref()?.node.as_ref()? {
        NodeEnum::ColumnRef(column) => last(&column.fields),
        NodeEnum::FuncCall(call) => last(&call.funcname),
        _ => None,
    }
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
