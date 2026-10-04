use super::*;

/// `$1` in the body of a SQL function is the type of its parameter. [`transformParamRef`]
/// asks the function's parameter hook for it.
///
/// [`transformParamRef`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L885
pub(super) fn infer_param_ref(
    r: &mut Resolver<'_>,
    n: &pgls_query::protobuf::ParamRef,
) -> Option<Type> {
    {
        let param = r
            .function?
            .params
            .get(usize::try_from(n.number).ok()?.checked_sub(1)?)?;
        function_param_type(r, param)
    }
}
