use pgls_query::protobuf::DeleteStmt;

use super::{
    Resolver, from_clause::resolve_from_clause, range_var::resolve_target_relation,
    res_target::resolve_returning_list, resolve_node, with_clause::resolve_with_clause,
};
use crate::resolve::scope::{Columns, Level};

/// Resolves a DELETE and returns the columns of its RETURNING list.
pub(super) fn resolve_delete_stmt(r: &mut Resolver, n: &DeleteStmt) -> Columns {
    let ctes = r.ctes.len();
    if let Some(with) = &n.with_clause {
        resolve_with_clause(r, with);
    }
    let columns = resolve_delete(r, n);
    r.ctes.truncate(ctes);
    columns
}

fn resolve_delete(r: &mut Resolver, n: &DeleteStmt) -> Columns {
    let Some(relation) = &n.relation else {
        r.depends_on_file();
        return None;
    };
    let target = resolve_target_relation(r, relation);

    let mut level = Level::with_item(target);
    level.extend(resolve_from_clause(r, &n.using_clause));
    r.enter_level(level);

    if let Some(where_clause) = n.where_clause.as_deref() {
        resolve_node(r, where_clause);
        if let Some(expr) = where_clause.node.as_ref() {
            super::super::expr::infer_expr(r, expr);
        }
    }

    let level = r.exit_level();
    resolve_returning_list(r, &n.returning_list, level)
}
