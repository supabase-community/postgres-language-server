use pgls_query::protobuf::RangeSubselect;

use super::{Resolver, alias::apply_alias, resolve_node};
use crate::resolve::scope::{Item, Level};

/// A subquery in FROM. Only a LATERAL subquery sees the FROM items before it.
pub(super) fn resolve_range_subselect(
    r: &mut Resolver,
    n: &RangeSubselect,
    preceding: &Level,
) -> Item {
    let columns = n.subquery.as_deref().and_then(|query| {
        if !n.lateral {
            return resolve_node(r, query);
        }
        r.enter_level(preceding.clone());
        let columns = resolve_node(r, query);
        r.exit_level();
        columns
    });
    let alias = n.alias.as_ref();
    Item::named(
        alias.map(|alias| alias.aliasname.clone()),
        apply_alias(columns, alias),
    )
}
