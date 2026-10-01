use super::*;

pub(super) fn infer_collate_clause(
    r: &mut Resolver<'_>,
    n: &pgls_query::protobuf::CollateClause,
) -> Option<Type> {
    n.arg
        .as_deref()
        .and_then(|x| x.node.as_ref())
        .and_then(|x| infer_expr(r, x))
}
