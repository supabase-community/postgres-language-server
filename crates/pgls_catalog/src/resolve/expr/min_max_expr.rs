use super::*;

/// `GREATEST` and `LEAST` are the common type of their arguments. Port of
/// [`transformMinMaxExpr`].
///
/// [`transformMinMaxExpr`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L2275
pub(super) fn infer_min_max_expr(
    r: &mut Resolver<'_>,
    n: &pgls_query::protobuf::MinMaxExpr,
) -> Option<Type> {
    common(r, &n.args)
}
