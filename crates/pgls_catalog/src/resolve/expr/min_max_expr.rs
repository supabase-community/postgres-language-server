use super::*;

pub(super) fn infer_min_max_expr(
    r: &mut Resolver<'_>,
    n: &pgls_query::protobuf::MinMaxExpr,
) -> Option<Type> {
    common(r, &n.args)
}
