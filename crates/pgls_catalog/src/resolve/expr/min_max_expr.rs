use super::*;

/// `GREATEST` and `LEAST` are the common type of their arguments, which are coerced to it. Port
/// of [`transformMinMaxExpr`].
///
/// [`transformMinMaxExpr`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L2275
pub(super) fn infer_min_max_expr(
    r: &mut Resolver<'_>,
    n: &pgls_query::protobuf::MinMaxExpr,
) -> Option<Type> {
    let common = common(r, &n.args);
    record_enum_values(
        r,
        n.args.iter().map(|arg| arg.node.as_ref()),
        common.as_ref(),
    );
    common
}
