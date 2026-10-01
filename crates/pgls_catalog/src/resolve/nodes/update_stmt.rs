use pgls_query::{NodeEnum, protobuf::UpdateStmt};

use super::{
    Resolver,
    from_clause::resolve_from_clause,
    range_var::resolve_target_relation,
    res_target::{check_target_column, resolve_returning_list},
    resolve_node,
    with_clause::resolve_with_clause,
};
use crate::resolve::scope::{Columns, Level};

/// Resolves an UPDATE and returns the columns of its RETURNING list.
pub(super) fn resolve_update_stmt(r: &mut Resolver, n: &UpdateStmt) -> Columns {
    let ctes = r.ctes.len();
    if let Some(with) = &n.with_clause {
        resolve_with_clause(r, with);
    }
    let columns = resolve_update(r, n);
    r.ctes.truncate(ctes);
    columns
}

fn resolve_update(r: &mut Resolver, n: &UpdateStmt) -> Columns {
    let Some(relation) = &n.relation else {
        r.depends_on_file();
        return None;
    };
    let target = resolve_target_relation(r, relation);

    let mut level = Level::with_item(target.clone());
    level.extend(resolve_from_clause(r, &n.from_clause));
    r.enter_level(level);

    for assignment in &n.target_list {
        if let Some(NodeEnum::ResTarget(assignment)) = &assignment.node {
            check_target_column(r, &target, &assignment.name, assignment.location);
            // Assignments to subscripts or fields, like `a[1]` or `c.f`, aren't checked.
            if let (true, Some(column_type), Some(expr)) = (
                assignment.indirection.is_empty(),
                target.type_of(&assignment.name).flatten(),
                assignment.val.as_deref().and_then(|v| v.node.as_ref()),
            ) {
                let expr_type = super::super::expr::infer_expr(r, expr);
                super::super::expr::check_assignment(
                    r,
                    &assignment.name,
                    column_type,
                    expr_type,
                    assignment
                        .val
                        .as_deref()
                        .and_then(super::insert_stmt::node_location)
                        .unwrap_or(assignment.location),
                );
            }
        }
        resolve_node(r, assignment);
    }
    if let Some(where_clause) = n.where_clause.as_deref() {
        resolve_node(r, where_clause);
        if let Some(expr) = where_clause.node.as_ref() {
            super::super::expr::infer_expr(r, expr);
        }
    }

    let level = r.exit_level();
    resolve_returning_list(r, &n.returning_list, level)
}
