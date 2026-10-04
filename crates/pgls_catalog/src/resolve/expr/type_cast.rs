use super::*;

/// The target type of a cast, and a finding when no explicit coercion exists. Port of
/// [`transformTypeCast`], which coerces with [`can_coerce_type`] in the explicit context.
///
/// [`transformTypeCast`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L2714
/// [`can_coerce_type`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_coerce.c#L557
pub(super) fn infer_type_cast(
    r: &mut Resolver<'_>,
    n: &pgls_query::protobuf::TypeCast,
) -> Option<Type> {
    {
        let from = n
            .arg
            .as_deref()?
            .node
            .as_ref()
            .and_then(|x| infer_expr(r, x));
        let (id, _) = normalize_type_name(r.catalog, n.type_name.as_ref()?, r.search_path);
        if let (Some(from), Some(to)) = (from, id.as_ref())
            && !matches!(from, Type::UnknownLiteral)
            && matches!(
                can_coerce(r.catalog, &from, to, CoercionContext::Explicit),
                Decision::Known(false)
            )
        {
            let span = r.cast_span(n.location);
            r.report_with_span(
                crate::resolve::FindingKind::InvalidCast {
                    from,
                    to: Type::Named(to.clone()),
                },
                span,
            );
            return None;
        }
        id.map(Type::Named)
    }
}
