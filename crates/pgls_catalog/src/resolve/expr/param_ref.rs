use super::*;

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
