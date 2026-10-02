use pgls_query::protobuf::RangeSubselect;

use super::{Resolver, alias::apply_alias, resolve_node};
use crate::resolve::scope::{Item, Level};

/// A subquery in FROM. Only a LATERAL subquery sees the FROM items before it.
pub(super) fn resolve_range_subselect(
    r: &mut Resolver,
    n: &RangeSubselect,
    preceding: &Level,
) -> Item {
    let mut typed_columns = None;
    let columns = n.subquery.as_deref().and_then(|query| {
        let columns = if !n.lateral {
            resolve_node(r, query)
        } else {
            r.enter_level(preceding.clone());
            let columns = resolve_node(r, query);
            r.exit_level();
            columns
        };
        typed_columns = crate::resolve::resolve_unknown_outputs(r.catalog, r.output.clone());
        columns
    });
    let alias = n.alias.as_ref();
    let mut item = Item::named(
        alias.map(|alias| alias.aliasname.clone()),
        apply_alias(columns, alias),
    );
    item.typed_columns = typed_columns.map(|mut columns| {
        if let Some(alias) = alias {
            if let Some(names) = super::string::string_values(&alias.colnames) {
                for (column, name) in columns.iter_mut().zip(names) {
                    column.name = name;
                }
            }
        }
        columns
    });
    item
}
