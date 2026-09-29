//! Queries: SELECT, set operations, VALUES, and WITH.

use pgls_query::{NodeEnum, protobuf};

use super::{
    Ctes, Resolver,
    expr::ExprOptions,
    from::with_level,
    scope::{Columns, Cte, Level},
    string_value, string_values,
};
use crate::column_name::figure_column_name;

impl Resolver<'_> {
    /// Resolves a query and returns its output columns, if known.
    pub(super) fn query(
        &mut self,
        query: &protobuf::Node,
        outer: &[&Level],
        ctes: Ctes,
    ) -> Columns {
        match query.node.as_ref() {
            Some(NodeEnum::SelectStmt(select)) => self.select(select, outer, ctes),
            // Data-modifying statements in WITH.
            Some(NodeEnum::InsertStmt(insert)) => self.insert(insert, ctes),
            Some(NodeEnum::UpdateStmt(update)) => self.update(update, ctes),
            Some(NodeEnum::DeleteStmt(delete)) => self.delete(delete, ctes),
            _ => {
                self.depends_on_file();
                None
            }
        }
    }

    pub(super) fn select(
        &mut self,
        select: &protobuf::SelectStmt,
        outer: &[&Level],
        ctes: Ctes,
    ) -> Columns {
        let with_ctes;
        let ctes = match &select.with_clause {
            Some(with) => {
                with_ctes = self.with_clause(with, outer, ctes);
                with_ctes.as_slice()
            }
            None => ctes,
        };

        // Set operations take their column names from the left-most query. Their ORDER BY and
        // LIMIT reference the output columns and are not checked.
        if select.op() != protobuf::SetOperation::SetopNone {
            let columns = select
                .larg
                .as_deref()
                .and_then(|left| self.select(left, outer, ctes));
            if let Some(right) = select.rarg.as_deref() {
                self.select(right, outer, ctes);
            }
            return columns;
        }

        if !select.values_lists.is_empty() {
            let mut width = None;
            for row in &select.values_lists {
                if let Some(NodeEnum::List(row)) = &row.node {
                    width.get_or_insert(row.items.len());
                    for value in &row.items {
                        self.expr(value, outer, ctes, ExprOptions::default());
                    }
                }
            }
            return width.map(|width| (1..=width).map(|i| format!("column{i}")).collect());
        }

        let mut level = self.resolve_from(&select.from_clause, outer, ctes);
        level.output_names = select
            .target_list
            .iter()
            .filter_map(|target| match &target.node {
                Some(NodeEnum::ResTarget(target)) if !target.name.is_empty() => {
                    Some(target.name.clone())
                }
                Some(NodeEnum::ResTarget(target)) => target
                    .val
                    .as_deref()
                    .and_then(|value| value.node.as_ref())
                    .and_then(figure_column_name),
                _ => None,
            })
            .collect();
        let levels = with_level(outer, &level);

        let plain = ExprOptions::default();
        let with_output_names = ExprOptions { output_names: true };
        for target in &select.target_list {
            self.expr(target, &levels, ctes, plain);
        }
        for node in select.where_clause.iter().chain(&select.having_clause) {
            self.expr(node, &levels, ctes, plain);
        }
        for node in select
            .group_clause
            .iter()
            .chain(&select.sort_clause)
            .chain(&select.distinct_clause)
        {
            self.expr(node, &levels, ctes, with_output_names);
        }
        for node in &select.window_clause {
            self.expr(node, &levels, ctes, plain);
        }
        for node in select.limit_offset.iter().chain(&select.limit_count) {
            self.expr(node, &levels, ctes, plain);
        }

        self.target_columns(&select.target_list, &levels)
    }

    /// Resolves the CTEs of a WITH clause and returns the CTEs visible to its query.
    pub(super) fn with_clause(
        &mut self,
        with: &protobuf::WithClause,
        outer: &[&Level],
        ctes: Ctes,
    ) -> Vec<Cte> {
        let mut visible: Vec<Cte> = ctes.to_vec();
        let definitions: Vec<&protobuf::CommonTableExpr> = with
            .ctes
            .iter()
            .filter_map(|node| match &node.node {
                Some(NodeEnum::CommonTableExpr(cte)) => Some(cte.as_ref()),
                _ => None,
            })
            .collect();

        // In WITH RECURSIVE, every CTE can reference every CTE of the list, including itself.
        if with.recursive {
            visible.extend(definitions.iter().map(|cte| Cte {
                name: cte.ctename.clone(),
                columns: None,
            }));
        }

        for cte in definitions {
            let columns = cte
                .ctequery
                .as_deref()
                .and_then(|query| self.query(query, outer, &visible));
            let columns = rename_columns(columns, &cte.aliascolnames);
            // SEARCH and CYCLE add columns.
            let columns = if cte.search_clause.is_some() || cte.cycle_clause.is_some() {
                None
            } else {
                columns
            };

            let resolved = Cte {
                name: cte.ctename.clone(),
                columns,
            };
            match visible
                .iter_mut()
                .rev()
                .find(|visible| with.recursive && visible.name == cte.ctename)
            {
                Some(existing) => *existing = resolved,
                None => visible.push(resolved),
            }
        }

        visible
    }

    /// The output columns of a select list or RETURNING list.
    pub(super) fn target_columns(&self, targets: &[protobuf::Node], levels: &[&Level]) -> Columns {
        let level = levels.last()?;
        let mut columns = Vec::new();
        for target in targets {
            let NodeEnum::ResTarget(target) = target.node.as_ref()? else {
                return None;
            };
            let value = target
                .val
                .as_deref()
                .and_then(|value| value.node.as_ref())?;

            if let NodeEnum::ColumnRef(column) = value {
                if let Some((last, qualifier)) = column.fields.split_last() {
                    if matches!(last.node, Some(NodeEnum::AStar(_))) {
                        let expanded = match qualifier {
                            [] => level.star_columns()?,
                            [name] => level.item(string_value(name)?)?.columns.clone()?,
                            _ => return None,
                        };
                        columns.extend(expanded);
                        continue;
                    }
                }
            }

            if !target.name.is_empty() {
                columns.push(target.name.clone());
            } else {
                columns.push(figure_column_name(value)?);
            }
        }
        Some(columns)
    }
}

/// Applies a column name list (`WITH c(a, b) AS ...`) to the first columns.
fn rename_columns(columns: Columns, names: &[protobuf::Node]) -> Columns {
    if names.is_empty() {
        return columns;
    }
    let mut columns = columns?;
    let names = string_values(names)?;
    if names.len() > columns.len() {
        return None;
    }
    for (column, name) in columns.iter_mut().zip(names) {
        *column = name;
    }
    Some(columns)
}
