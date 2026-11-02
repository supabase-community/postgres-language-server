use pgls_query::protobuf::UpdateStmt;

use super::{
    Resolver,
    from_clause::resolve_from_clause,
    range_var::resolve_target_relation,
    res_target::{resolve_returning_clause, resolve_set_clause},
    resolve_node,
    with_clause::resolve_with_clause,
};
use crate::resolve::scope::{Columns, Level};

/// Resolves an UPDATE and returns the columns of its RETURNING list. Port of
/// [`transformUpdateStmt`] and [`transformUpdateTargetList`].
///
/// [`transformUpdateStmt`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/analyze.c#L2472
/// [`transformUpdateTargetList`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/analyze.c#L2546
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

    resolve_set_clause(r, &target, &n.target_list);
    if let Some(where_clause) = n.where_clause.as_deref() {
        resolve_node(r, where_clause);
        if let Some(expr) = where_clause.node.as_ref() {
            super::super::expr::infer_expr(r, expr);
        }
    }

    let level = r.exit_level();
    resolve_returning_clause(r, n.returning_clause.as_ref(), level)
}
