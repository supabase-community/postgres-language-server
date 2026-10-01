use super::*;

pub(super) fn infer_row_expr(
    r: &mut Resolver<'_>,
    n: &pgls_query::protobuf::RowExpr,
) -> Option<Type> {
    Some(Type::Record(
        n.args
            .iter()
            .enumerate()
            .map(|(i, arg)| crate::typing::TypedColumn {
                name: n
                    .colnames
                    .get(i)
                    .and_then(string_value)
                    .unwrap_or("")
                    .to_owned(),
                ty: arg.node.as_ref().and_then(|x| infer_expr(r, x)),
            })
            .collect(),
    ))
}
