use super::*;

/// `AND`, `OR` and `NOT` are bool. Port of [`transformBoolExpr`].
///
/// [`transformBoolExpr`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L1413
pub(super) fn infer_bool_expr(
    r: &mut Resolver<'_>,
    n: &pgls_query::protobuf::BoolExpr,
) -> Option<Type> {
    {
        for arg in &n.args {
            if let Some(arg) = arg.node.as_ref() {
                infer_expr(r, arg);
            }
        }
        named(r, "bool")
    }
}
