use super::*;

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
