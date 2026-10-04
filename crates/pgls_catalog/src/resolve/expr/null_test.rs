use super::*;

/// `IS [NOT] NULL` is bool (the `T_NullTest` case of [`transformExprRecurse`]).
///
/// [`transformExprRecurse`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L137
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
