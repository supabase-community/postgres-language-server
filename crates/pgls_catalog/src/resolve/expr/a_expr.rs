use super::*;

pub(super) fn infer_a_expr(r: &mut Resolver<'_>, n: &pgls_query::protobuf::AExpr) -> Option<Type> {
    {
        use protobuf::AExprKind as Kind;
        match Kind::try_from(n.kind).ok()? {
            // Port of parse_expr.c `transformAExprDistinct`; ordinary scalar cases use make_distinct_op's `=` lookup.
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
            // Port of parse_expr.c `transformAExprOpAny`/`transformAExprOpAll`; parse_oper.c `make_scalar_array_op` uses the array element type.
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
            // Port of parse_expr.c `transformAExprIn`: try a common scalar type, then compare items individually.
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
                    .filter_map(|item| infer_expr(r, item))
                    .collect::<Vec<_>>();
                if let Some(left) = left.as_ref() {
                    if items.len() > 1 && types.len() == items.len() {
                        // Port of transformAExprIn: common-type array optimization for non-Var items.
                        if let Selection::Match(common) = select_common_type(
                            r.catalog,
                            &[std::iter::once(left.clone())
                                .chain(types.clone())
                                .collect::<Vec<_>>()]
                            .concat(),
                        ) {
                            if !matches!(common, Type::Record(_)) {
                                check_operator(r, name, Some(left), &common, n.location);
                                return named(r, "bool");
                            }
                        }
                    }
                    for ty in &types {
                        check_operator(r, name, Some(left), ty, n.location);
                    }
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
                            // Port of parse_expr.c `transformAExprBetween`.
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
            // Port of parse_expr.c `transformAExprOp`: LIKE/ILIKE lower to ~~/~~*/!~~/!~~*; SIMILAR is conservatively checked as ~.
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
            // Port of parse_expr.c `transformAExprNullIf`, which selects `=` with make_op.
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
        let left = n
            .lexpr
            .as_deref()
            .and_then(|x| x.node.as_ref())
            .and_then(|x| infer_expr(r, x));
        let kind = if left.is_some() {
            OperatorKind::Infix
        } else {
            OperatorKind::Prefix
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
