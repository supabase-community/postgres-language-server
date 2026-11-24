use std::rc::Rc;

use pgls_query::protobuf::RangeSubselect;

use super::{Resolver, alias::apply_alias, resolve_node};
use crate::{
    ColumnSource,
    output::QueryOutput,
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
    let output = n.subquery.as_deref().and_then(|query| {
        if !n.lateral {
            resolve_node(r, query)
        } else {
            let (output, _) = r.in_level(preceding.clone(), |r| resolve_node(r, query));
            output
        }
    });
    let columns = pgls_catalog::typing::resolve_unknown_outputs(
        r.catalog,
        output.as_ref().and_then(QueryOutput::columns),
    );
    let types_known = columns.is_some();
    let alias = n.alias.as_ref();
    let source = match &columns {
        Some(columns) if r.collects_references() => Some(Rc::new(ItemSource::derived(
            r,
            ColumnSource::Subquery {
                alias: alias.map(|alias| alias.aliasname.clone()),
            },
            columns,
            types_known,
        ))),
        _ => None,
    };
    let mut item = Item::named(
        alias.map(|alias| alias.aliasname.clone()),
        apply_alias(columns, alias),
        types_known,
    );
    item.source = source;
    item
}
