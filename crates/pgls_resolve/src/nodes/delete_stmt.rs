use pgls_query::protobuf::DeleteStmt;

use super::{
    Resolver, from_clause::resolve_from_clause, range_var::resolve_target_relation,
    res_target::resolve_returning_clause, with_clause::resolve_with_clause,
};
use crate::{output::QueryOutput, scope::Level, utils::resolve_expr};

/// Resolves a DELETE and returns the columns of its RETURNING list. Port of
/// [`transformDeleteStmt`].
///
/// [`transformDeleteStmt`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/analyze.c#L553
pub(super) fn resolve_delete_stmt(r: &mut Resolver, n: &DeleteStmt) -> Option<QueryOutput> {
    r.with_ctes(|r| {
        if let Some(with) = &n.with_clause {
            resolve_with_clause(r, with);
        }
        resolve_delete(r, n)
    })
}

fn resolve_delete(r: &mut Resolver, n: &DeleteStmt) -> Option<QueryOutput> {
    let Some(relation) = &n.relation else {
        r.depends_on_file();
        return None;
    };
    let target = resolve_target_relation(r, relation);

    let mut level = Level::with_item(target.clone());
    level.extend(resolve_from_clause(r, &n.using_clause));
    let ((), level) = r.in_level(level, |r| {
        if let Some(where_clause) = n.where_clause.as_deref() {
            resolve_expr(r, where_clause);
        }
    });
    resolve_returning_clause(r, n.returning_clause.as_ref(), &target, level)
}
