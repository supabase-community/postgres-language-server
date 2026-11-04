use std::rc::Rc;

use pgls_query::protobuf::RangeSubselect;

use super::{Resolver, alias::apply_alias, resolve_node};
use crate::resolve::{
    ColumnSource,
    scope::{Item, ItemSource, Level},
};

/// A subquery in FROM. Only a LATERAL subquery sees the FROM items before it. Port of
/// [`transformRangeSubselect`].
///
/// [`transformRangeSubselect`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_clause.c#L405
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
    let source = match &columns {
        Some(names) if r.collects_references() => Some(Rc::new(ItemSource::derived(
            r,
            ColumnSource::Subquery {
                alias: alias.map(|alias| alias.aliasname.clone()),
            },
            names,
            typed_columns.as_deref(),
        ))),
        _ => None,
    };
    let mut item = Item::named(
        alias.map(|alias| alias.aliasname.clone()),
        apply_alias(columns, alias),
    );
    item.typed_columns = typed_columns.map(|mut columns| {
        if let Some(alias) = alias
            && let Some(names) = pgls_query_ext::utils::string_values(&alias.colnames)
        {
            for (column, name) in columns.iter_mut().zip(names) {
                column.name = name;
            }
        }
        columns
    });
    item.source = source;
    item
}
