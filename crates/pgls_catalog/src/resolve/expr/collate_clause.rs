use super::*;

/// `COLLATE` keeps the type of its argument. Port of [`transformCollateClause`].
///
/// [`transformCollateClause`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L2798
pub(super) fn infer_collate_clause(
    r: &mut Resolver<'_>,
    n: &pgls_query::protobuf::CollateClause,
) -> Option<Type> {
    n.arg
        .as_deref()
        .and_then(|x| x.node.as_ref())
        .and_then(|x| infer_expr(r, x))
}
