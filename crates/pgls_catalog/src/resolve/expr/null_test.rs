use super::*;

pub(super) fn infer_null_test(
    r: &mut Resolver<'_>,
    n: &pgls_query::protobuf::NullTest,
) -> Option<Type> {
    {
        if let Some(arg) = n.arg.as_deref().and_then(|x| x.node.as_ref()) {
            infer_expr(r, arg);
        }
        named(r, "bool")
    }
}
