use super::*;

/// `EXISTS`, `ANY`/`ALL` and `IN` subqueries are bool, a scalar subquery is the type of its only
/// column. Port of [`transformSubLink`].
///
/// [`transformSubLink`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L1782
pub(super) fn infer_sub_link(
    r: &mut Resolver<'_>,
    n: &pgls_query::protobuf::SubLink,
) -> Option<Type> {
    {
        if let Some(test) = n.testexpr.as_deref().and_then(|x| x.node.as_ref()) {
            infer_expr(r, test);
        }
        match protobuf::SubLinkType::try_from(n.sub_link_type).ok()? {
            protobuf::SubLinkType::ExistsSublink => named(r, "bool"),
            protobuf::SubLinkType::ExprSublink => {
                let query = n.subselect.as_deref()?;
                let columns = crate::resolve::query_output_types(query, r.catalog, r.search_path)?;
                (columns.len() == 1)
                    .then(|| columns[0].ty.clone())
                    .flatten()
            }
            protobuf::SubLinkType::AnySublink | protobuf::SubLinkType::AllSublink => {
                named(r, "bool")
            }
            _ => None,
        }
    }
}
