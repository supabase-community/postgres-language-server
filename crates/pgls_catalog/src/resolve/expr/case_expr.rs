use super::*;

/// `CASE` is the common type of its results. Port of [`transformCaseExpr`], which also
/// compares the test value of a simple `CASE` with each `WHEN` value using `=`.
///
/// [`transformCaseExpr`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L1642
pub(super) fn infer_case_expr(
    r: &mut Resolver<'_>,
    n: &pgls_query::protobuf::CaseExpr,
) -> Option<Type> {
    {
        let test_type = n
            .arg
            .as_deref()
            .and_then(|x| x.node.as_ref())
            .and_then(|x| infer_expr(r, x));
        let mut results = Vec::new();
        for when in &n.args {
            let Some(NodeEnum::CaseWhen(when)) = when.node.as_ref() else {
                return None;
            };
            let when_type = when
                .expr
                .as_deref()
                .and_then(|x| x.node.as_ref())
                .and_then(|x| infer_expr(r, x));
            if let (Some(left), Some(right)) = (test_type.as_ref(), when_type.as_ref()) {
                // Port of the `=` comparison `transformCaseExpr` builds with `make_op`; unknown
                // candidate metadata deliberately suppresses a finding:
                // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L1642
                let comparison = select_operator(
                    r.catalog,
                    None,
                    "=",
                    OperatorKind::Infix,
                    Some(left),
                    right,
                    r.search_path,
                );
                let failure = match comparison {
                    Selection::NoMatch => Some(crate::resolve::MatchFailure::NoMatch),
                    Selection::Ambiguous => Some(crate::resolve::MatchFailure::Ambiguous),
                    _ => None,
                };
                if let Some(failure) = failure {
                    r.report(
                        crate::resolve::FindingKind::OperatorMismatch {
                            operator: "=".to_owned(),
                            left: Some(left.clone()),
                            right: right.clone(),
                            failure,
                        },
                        when.location,
                    );
                }
            }
            results.push(
                when.result
                    .as_deref()
                    .and_then(|x| x.node.as_ref())
                    .and_then(|x| infer_expr(r, x)),
            );
        }
        // Without ELSE, the result is NULL otherwise.
        results.push(match n.defresult.as_deref().and_then(|x| x.node.as_ref()) {
            Some(default) => infer_expr(r, default),
            None => Some(Type::UnknownLiteral),
        });
        common_types(r, &results.into_iter().collect::<Option<Vec<_>>>()?)
    }
}
