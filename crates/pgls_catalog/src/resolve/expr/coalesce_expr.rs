use super::*;

pub(super) fn infer_coalesce_expr(
    r: &mut Resolver<'_>,
    n: &pgls_query::protobuf::CoalesceExpr,
) -> Option<Type> {
    common(r, &n.args)
}
