use super::*;

/// `ARRAY[...]`: the array type of the elements' common type. Port of [`transformArrayExpr`]
/// for one-dimensional arrays with a known element type.
///
/// [`transformArrayExpr`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L2025
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
