use std::rc::Rc;

use pgls_query::protobuf::RangeSubselect;

use super::{Resolver, alias::apply_alias, resolve_node};
use crate::resolve::{
    ColumnSource,
    output::QueryOutput,
    scope::{Item, ItemSource, Level, combine_columns},
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
            r.enter_level(preceding.clone());
            let output = resolve_node(r, query);
            r.exit_level();
            output
        }
    });
    let (columns, types_known) = combine_columns(
        output.as_ref().and_then(QueryOutput::names),
        crate::resolve::resolve_unknown_outputs(
            r.catalog,
            output.as_ref().and_then(QueryOutput::columns),
        ),
    );
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
