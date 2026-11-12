use pgls_query::protobuf::OnConflictClause;

use super::{Resolver, res_target::resolve_set_clause, resolve_node};
use crate::resolve::scope::{Item, Level};

/// `ON CONFLICT` sees the target relation and the row proposed for insertion, `excluded`. Port
/// of [`transformOnConflictClause`].
///
/// [`transformOnConflictClause`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/analyze.c#L1170
pub(super) fn resolve_on_conflict_clause(r: &mut Resolver, n: &OnConflictClause, target: &Item) {
    // The arbiter's index predicate sees only the target relation, like in
    // `transformOnConflictArbiter`: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_clause.c#L3297
    if let Some(where_clause) = n
        .infer
        .as_ref()
        .and_then(|infer| infer.where_clause.as_deref())
    {
        r.enter_level(Level {
            items: vec![target.clone()],
            ..Default::default()
        });
        resolve_node(r, where_clause);
        r.exit_level();
    }

    let excluded = Item::named(Some("excluded".into()), target.columns.clone());
    r.enter_level(Level {
        items: vec![target.clone(), excluded],
        ..Default::default()
    });
    resolve_set_clause(r, target, &n.target_list);
    if let Some(where_clause) = n.where_clause.as_deref() {
        resolve_node(r, where_clause);
    }

    r.exit_level();
}
