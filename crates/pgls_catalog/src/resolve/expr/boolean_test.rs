use super::*;

/// `IS [NOT] TRUE/FALSE/UNKNOWN` is bool. Port of [`transformBooleanTest`].
///
/// [`transformBooleanTest`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L2540
pub(super) fn infer_boolean_test(
    r: &mut Resolver<'_>,
    n: &pgls_query::protobuf::BooleanTest,
) -> Option<Type> {
    {
        if let Some(arg) = n.arg.as_deref().and_then(|x| x.node.as_ref()) {
            infer_expr(r, arg);
        }
        named(r, "bool")
    }
}
