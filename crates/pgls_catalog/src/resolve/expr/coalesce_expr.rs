use super::*;

/// `COALESCE` is the common type of its arguments. Port of [`transformCoalesceExpr`].
///
/// [`transformCoalesceExpr`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L2226
pub(super) fn infer_coalesce_expr(
    r: &mut Resolver<'_>,
    n: &pgls_query::protobuf::CoalesceExpr,
) -> Option<Type> {
    common(r, &n.args)
}
