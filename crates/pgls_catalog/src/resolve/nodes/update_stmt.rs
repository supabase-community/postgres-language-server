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
        }
        resolve_node(r, assignment);
    }
    if let Some(where_clause) = n.where_clause.as_deref() {
        resolve_node(r, where_clause);
    }

    let level = r.exit_level();
    resolve_returning_list(r, &n.returning_list, level)
}
