use super::*;

/// Operators, `IS DISTINCT FROM`, `= ANY`, `IN`, `BETWEEN`, `LIKE` and `NULLIF`, which
/// [`transformExprRecurse`] dispatches on the kind of the `A_Expr`.
///
/// [`transformExprRecurse`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L137
pub(super) fn infer_a_expr(r: &mut Resolver<'_>, n: &pgls_query::protobuf::AExpr) -> Option<Type> {
    {
        use protobuf::AExprKind as Kind;
        match Kind::try_from(n.kind).ok()? {
            // Port of `transformAExprDistinct`; scalar cases look up `=` like `make_distinct_op`:
            // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L1031
            // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L3084
            Kind::AexprDistinct | Kind::AexprNotDistinct => {
                let left = n
                    .lexpr
                    .as_deref()
                    .and_then(|x| x.node.as_ref())
                    .and_then(|x| infer_expr(r, x));
                let right = n
                    .rexpr
                    .as_deref()
                    .and_then(|x| x.node.as_ref())
                    .and_then(|x| infer_expr(r, x));
                if let (Some(left), Some(right)) = (left.as_ref(), right.as_ref()) {
                    check_operator(r, "=", Some(left), right, n.location);
                }
                return named(r, "bool");
            }
            // Port of `transformAExprOpAny` and `transformAExprOpAll`; `make_scalar_array_op` uses
            // the array element type:
            // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L1003
            // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_oper.c#L792
            Kind::AexprOpAny | Kind::AexprOpAll => {
                let left = n
                    .lexpr
                    .as_deref()
                    .and_then(|x| x.node.as_ref())
                    .and_then(|x| infer_expr(r, x));
                let array = n
                    .rexpr
                    .as_deref()
                    .and_then(|x| x.node.as_ref())
                    .and_then(|x| infer_expr(r, x));
                let element = match array {
                    Some(Type::UnknownLiteral) => Some(Type::UnknownLiteral),
                    Some(Type::Named(id)) => r
                        .catalog
                        .type_by_id(&id)
                        .found()
                        .and_then(|i| i.element)
                        .map(Type::Named),
                    _ => None,
                };
                let (Some(left), Some(element)) = (left, element) else {
                    return named(r, "bool");
                };
                let op = string_values(&n.name)?.last()?.clone();
                check_operator(r, &op, Some(&left), &element, n.location);
                return named(r, "bool");
            }
            // Port of `transformAExprIn`: try a common scalar type, then compare items one by one:
            // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L1125
            Kind::AexprIn => {
                let left = n
                    .lexpr
                    .as_deref()
                    .and_then(|x| x.node.as_ref())
                    .and_then(|x| infer_expr(r, x));
                let rhs = n.rexpr.as_deref().and_then(|x| x.node.as_ref());
                let items = match rhs {
                    Some(NodeEnum::List(list)) => list
                        .items
                        .iter()
                        .filter_map(|item| item.node.as_ref())
                        .collect::<Vec<_>>(),
                    Some(node) => vec![node],
                    None => Vec::new(),
                };
                let name = if string_values(&n.name)?.last()?.as_str() == "<>" {
                    "<>"
                } else {
                    "="
                };
                let types = items
                    .iter()
                    .map(|item| infer_expr(r, item))
                    .collect::<Vec<_>>();
                let (Some(left), Some(types)) =
                    (left.as_ref(), types.into_iter().collect::<Option<Vec<_>>>())
                else {
                    return named(r, "bool");
                };
                // Column references become `Var`s, which Postgres compares one by one. With more
                // than one other item, it tries a common type for them and the left side and
                // compares against an array of that type.
                let non_vars = items
                    .iter()
                    .zip(&types)
                    .filter(|(item, _)| !matches!(item, NodeEnum::ColumnRef(_)))
                    .map(|(_, ty)| ty.clone())
                    .collect::<Vec<_>>();
                if non_vars.len() > 1 {
                    let all = std::iter::once(left.clone())
                        .chain(non_vars)
                        .collect::<Vec<_>>();
                    match select_common_type(r.catalog, &all) {
                        Selection::Match(Type::Named(common)) => {
                            check_operator(r, name, Some(left), &Type::Named(common), n.location);
                            for (item, ty) in items.iter().zip(&types) {
                                if matches!(item, NodeEnum::ColumnRef(_)) {
                                    check_operator(r, name, Some(left), ty, n.location);
                                }
                            }
                            return named(r, "bool");
                        }
                        Selection::NoMatch | Selection::Ambiguous => {}
                        // A record common type also falls back; `unknown` is not modelled.
                        _ => return named(r, "bool"),
                    }
                }
                for ty in &types {
                    check_operator(r, name, Some(left), ty, n.location);
                }
                return named(r, "bool");
            }
            Kind::AexprBetween
            | Kind::AexprNotBetween
            | Kind::AexprBetweenSym
            | Kind::AexprNotBetweenSym => {
                let left = n
                    .lexpr
                    .as_deref()
                    .and_then(|x| x.node.as_ref())
                    .and_then(|x| infer_expr(r, x));
                let bounds = n.rexpr.as_deref().and_then(|x| x.node.as_ref());
                if let Some(NodeEnum::List(list)) = bounds {
                    if list.items.len() == 2 {
                        let low = list.items[0].node.as_ref().and_then(|x| infer_expr(r, x));
                        let high = list.items[1].node.as_ref().and_then(|x| infer_expr(r, x));
                        if let (Some(left), Some(low), Some(high)) =
                            (left.as_ref(), low.as_ref(), high.as_ref())
                        {
                            // Port of `transformAExprBetween`:
                            // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L1294
                            let kind = Kind::try_from(n.kind).ok()?;
                            let (lower, upper) = match kind {
                                Kind::AexprNotBetween | Kind::AexprNotBetweenSym => ("<", ">"),
                                _ => (">=", "<="),
                            };
                            check_operator(r, lower, Some(left), low, n.location);
                            check_operator(r, upper, Some(left), high, n.location);
                            if matches!(kind, Kind::AexprBetweenSym | Kind::AexprNotBetweenSym) {
                                check_operator(r, lower, Some(left), high, n.location);
                                check_operator(r, upper, Some(left), low, n.location);
                            }
                        }
                    }
                }
                return named(r, "bool");
            }
            // Port of `transformAExprOp`: LIKE/ILIKE lower to ~~/~~*/!~~/!~~*; SIMILAR is
            // conservatively checked as ~:
            // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L922
            Kind::AexprLike | Kind::AexprIlike | Kind::AexprSimilar => {
                let left = n
                    .lexpr
                    .as_deref()
                    .and_then(|x| x.node.as_ref())
                    .and_then(|x| infer_expr(r, x));
                let right = n
                    .rexpr
                    .as_deref()
                    .and_then(|x| x.node.as_ref())
                    .and_then(|x| infer_expr(r, x));
                if let (Some(left), Some(right)) = (left.as_ref(), right.as_ref()) {
                    let negated = string_values(&n.name)
                        .and_then(|v| v.last().cloned())
                        .as_deref()
                        == Some("!~~");
                    let op = match Kind::try_from(n.kind).ok()? {
                        Kind::AexprLike => {
                            if negated {
                                "!~~"
                            } else {
                                "~~"
                            }
                        }
                        Kind::AexprIlike => {
                            if negated {
                                "!~~*"
                            } else {
                                "~~*"
                            }
                        }
                        _ => "~",
                    };
                    check_operator(r, op, Some(left), right, n.location);
                }
                return named(r, "bool");
            }
            // Port of `transformAExprNullIf`, which selects `=` with `make_op`:
            // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L1082
            Kind::AexprNullif => {
                let left = n
                    .lexpr
                    .as_deref()?
                    .node
                    .as_ref()
                    .and_then(|x| infer_expr(r, x))?;
                let right = n
                    .rexpr
                    .as_deref()?
                    .node
                    .as_ref()
                    .and_then(|x| infer_expr(r, x))?;
                check_operator(r, "=", Some(&left), &right, n.location);
                return common_types(r, &[left, right]);
            }
            _ => {}
        }
        let names = string_values(&n.name)?;
        let (schema, name) = match names.as_slice() {
            [name] => (None, name.as_str()),
            [schema, name] => (Some(schema.as_str()), name.as_str()),
            _ => return None,
        };
        let right = infer_expr(r, n.rexpr.as_deref()?.node.as_ref()?)?;
        let (kind, left) = match n.lexpr.as_deref().and_then(|x| x.node.as_ref()) {
            Some(left) => (OperatorKind::Infix, Some(infer_expr(r, left)?)),
            None => (OperatorKind::Prefix, None),
        };
        let selection = select_operator(
            r.catalog,
            schema,
            name,
            kind,
            left.as_ref(),
            &right,
            r.search_path,
        );
        match selection {
            Selection::Match(o) => o.result,
            Selection::NoMatch | Selection::Ambiguous => {
                let failure = if matches!(selection, Selection::NoMatch) {
                    crate::resolve::MatchFailure::NoMatch
                } else {
                    crate::resolve::MatchFailure::Ambiguous
                };
                let span = r.operator_span(n.location);
                r.report_with_span(
                    crate::resolve::FindingKind::OperatorMismatch {
                        operator: name.to_owned(),
                        left,
                        right,
                        failure,
                    },
                    span,
                );
                None
            }
            _ => None,
        }
    }
}
