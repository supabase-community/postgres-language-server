use super::*;

pub(super) fn infer_a_array_expr(
    r: &mut Resolver<'_>,
    n: &pgls_query::protobuf::AArrayExpr,
) -> Option<Type> {
    let ty = common(r, &n.elements)?;
    let Type::Named(id) = ty else { return None };
    match r.catalog.type_by_id(&id) {
        Lookup::Found(info) => info.array.map(Type::Named),
        _ => None,
    }
}
