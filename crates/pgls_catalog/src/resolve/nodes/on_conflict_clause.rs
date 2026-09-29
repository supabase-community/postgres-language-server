use pgls_query::{NodeEnum, protobuf::OnConflictClause};

use super::{Resolver, res_target::check_target_column, resolve_node};
use crate::resolve::scope::{Item, Level};

/// `ON CONFLICT` sees the target relation and the row proposed for insertion, `excluded`.
pub(super) fn resolve_on_conflict_clause(r: &mut Resolver, n: &OnConflictClause, target: &Item) {
    let excluded = Item::named(Some("excluded".into()), target.columns.clone());
    r.enter_level(Level {
        items: vec![target.clone(), excluded],
        ..Default::default()
    });

    if let Some(where_clause) = n
        .infer
        .as_ref()
        .and_then(|infer| infer.where_clause.as_deref())
    {
        resolve_node(r, where_clause);
    }
    for assignment in &n.target_list {
        if let Some(NodeEnum::ResTarget(assignment)) = &assignment.node {
            check_target_column(r, target, &assignment.name, assignment.location);
        }
        resolve_node(r, assignment);
    }
    if let Some(where_clause) = n.where_clause.as_deref() {
        resolve_node(r, where_clause);
    }

    r.exit_level();
}
