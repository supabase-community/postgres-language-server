use crate::typing::{
    CallArg, CoercionContext, Decision, FunctionArgumentMode, OperatorKind, ResolvedFunction,
    ResolvedOperator, Selection, Type, TypeId, can_coerce, check_generic_type_consistency,
    resolve_polymorphic_result,
};
use crate::{CatalogView, FunctionInfo, lookup::Lookup};

fn poly(c: &dyn CatalogView, name: &str) -> Option<TypeId> {
    match c.type_(Some("pg_catalog"), name, &[]) {
        Lookup::Found(t) => t.id,
        _ => None,
    }
}
fn unknown(ty: &Type) -> bool {
    matches!(ty, Type::UnknownLiteral)
}
fn named(ty: &Type) -> Option<&TypeId> {
    if let Type::Named(id) = ty {
        Some(id)
    } else {
        None
    }
}
fn decision_coerce(c: &dyn CatalogView, from: &Type, to: &TypeId) -> Decision<bool> {
    can_coerce(c, from, to, CoercionContext::Implicit)
}

/// Port of `parse_oper.c:oper`, `binary_oper_exact`, and `oper_select_candidate`.
pub fn select_operator(
    c: &dyn CatalogView,
    schema: Option<&str>,
    name: &str,
    kind: OperatorKind,
    left: Option<&Type>,
    right: &Type,
    search_path: &[String],
) -> Selection<ResolvedOperator> {
    let candidates = c.operator_candidates(schema, name, kind, search_path);
    if candidates.items.is_empty() || !candidates.complete {
        return Selection::Unknown;
    }
    // `binary_oper_exact`: when one binary operand is unknown, first look for the other
    // operand's exact type on both sides of the operator.
    if kind == OperatorKind::Infix {
        let exact_side = if left.is_some_and(unknown) {
            right
        } else if unknown(right) {
            left.unwrap_or(right)
        } else {
            right
        };
        if let Some(id) = named(exact_side) {
            let exact: Vec<_> = candidates
                .items
                .iter()
                .filter(|op| op.left.as_ref() == Some(id) && op.right.as_ref() == Some(id))
                .collect();
            if exact.len() == 1 {
                let op = (*exact[0]).clone();
                return Selection::Match(ResolvedOperator {
                    result: op.result.clone().map(Type::Named),
                    operator: op,
                });
            }
        }
    }
    let mut viable = Vec::new();
    let mut indeterminate = false;
    for op in candidates.items {
        let Some(r) = op.right.as_ref() else {
            indeterminate = true;
            continue;
        };
        if op.kind != kind || (kind == OperatorKind::Infix) != op.left.is_some() {
            continue;
        }
        let mut args = Vec::new();
        let mut targets = Vec::new();
        if let Some(l) = left {
            let Some(t) = op.left.as_ref() else { continue };
            args.push(l);
            targets.push(t);
        }
        args.push(right);
        targets.push(r);
        let mut ok = true;
        let mut exact = true;
        for (arg, target) in args.iter().zip(&targets) {
            if named(arg) == Some(*target) {
                continue;
            }
            exact = false;
            if unknown(arg) {
                continue;
            }
            match decision_coerce(c, arg, target) {
                Decision::Known(true) => {}
                Decision::Known(false) => {
                    ok = false;
                    break;
                }
                Decision::Unknown => {
                    indeterminate = true;
                    ok = false;
                    break;
                }
            }
        }
        if ok {
            viable.push((op, exact));
        }
    }
    if indeterminate {
        return Selection::Unknown;
    }
    if viable.is_empty() {
        return Selection::NoMatch;
    }
    if unknown(right) || left.is_some_and(unknown) {
        let mut filtered = viable.clone();
        for (slot_unknown, left_slot) in
            [(left.is_some_and(unknown), true), (unknown(right), false)]
        {
            if !slot_unknown {
                continue;
            }
            let categories: Vec<_> = filtered
                .iter()
                .filter_map(|(op, _)| {
                    let id = if left_slot {
                        op.left.as_ref()?
                    } else {
                        op.right.as_ref()?
                    };
                    match c.type_by_id(id) {
                        Lookup::Found(t) => t.category,
                        _ => None,
                    }
                })
                .collect();
            if categories.len() != filtered.len() {
                return Selection::Unknown;
            }
            if categories.contains(&'S') {
                filtered.retain(|(op, _)| {
                    let id = if left_slot {
                        op.left.as_ref()
                    } else {
                        op.right.as_ref()
                    };
                    id.and_then(|id| match c.type_by_id(id) {
                        Lookup::Found(t) => t.category,
                        _ => None,
                    }) == Some('S')
                });
            }
            let preferred_exists = filtered.iter().any(|(op, _)| {
                let id = if left_slot {
                    op.left.as_ref()
                } else {
                    op.right.as_ref()
                };
                id.is_some_and(
                    |id| matches!(c.type_by_id(id), Lookup::Found(t) if t.preferred == Some(true)),
                )
            });
            if preferred_exists {
                filtered.retain(|(op, _)| {
                    let id = if left_slot { op.left.as_ref() } else { op.right.as_ref() };
                    id.is_some_and(|id| matches!(c.type_by_id(id), Lookup::Found(t) if t.preferred == Some(true)))
                });
            }
        }
        viable = filtered;
    }
    let exact: Vec<_> = viable.iter().filter(|(_, exact)| *exact).collect();
    let op = if exact.len() == 1 {
        exact[0].0.clone()
    } else if exact.len() > 1 {
        return Selection::Unknown;
    } else if viable.len() == 1 {
        viable[0].0.clone()
    } else {
        let max = viable
            .iter()
            .map(|(op, _)| {
                usize::from(left.and_then(named) == op.left.as_ref())
                    + usize::from(named(right) == op.right.as_ref())
            })
            .max()
            .unwrap_or(0);
        let best: Vec<_> = viable
            .iter()
            .filter(|(op, _)| {
                usize::from(left.and_then(named) == op.left.as_ref())
                    + usize::from(named(right) == op.right.as_ref())
                    == max
            })
            .collect();
        if best.len() != 1 {
            return Selection::Unknown;
        }
        best[0].0.clone()
    };
    Selection::Match(ResolvedOperator {
        result: op.result.clone().map(Type::Named),
        operator: op,
    })
}

fn input_arguments(
    info: &FunctionInfo,
) -> Option<Vec<(Option<String>, TypeId, FunctionArgumentMode)>> {
    let sig = info.signature.as_ref()?;
    let mut args = Vec::new();
    for arg in &sig.arguments {
        if arg.mode == FunctionArgumentMode::InOut {
            continue;
        }
        args.push((arg.name.clone(), arg.ty.clone()?, arg.mode));
    }
    Some(args)
}
fn arrange_args(info: &FunctionInfo, call: &[CallArg]) -> Option<(Vec<Type>, Vec<TypeId>)> {
    let sig = info.signature.as_ref()?;
    let params = input_arguments(info)?;
    let variadic = sig.variadic_element.is_some();
    let fixed = if variadic {
        params.len().saturating_sub(1)
    } else {
        params.len()
    };
    if call.len() < info.min_args || (!variadic && call.len() > params.len()) {
        return None;
    }
    if call.iter().any(|arg| arg.ty.is_none()) {
        return None;
    }
    let mut assigned: Vec<Option<Type>> = vec![None; params.len()];
    let mut positional = 0;
    let mut expanded_variadic = Vec::new();
    for arg in call {
        let index = if let Some(name) = &arg.name {
            params
                .iter()
                .position(|(parameter, _, _)| parameter.as_deref() == Some(name))?
        } else {
            let i = positional;
            positional += 1;
            if variadic && i >= fixed {
                expanded_variadic.push(arg.ty.clone()?);
                continue;
            }
            i
        };
        if index >= params.len() || assigned[index].is_some() {
            return None;
        }
        assigned[index] = arg.ty.clone();
    }
    if assigned.iter().enumerate().any(|(i, v)| {
        v.is_none() && i < params.len().saturating_sub(sig.input_defaults) && i < fixed
    }) {
        return None;
    }
    let mut types = Vec::new();
    let mut declared = Vec::new();
    for (i, (_, ty, mode)) in params.iter().enumerate() {
        if let Some(actual) = assigned[i].clone() {
            types.push(actual);
            declared.push(if *mode == FunctionArgumentMode::Variadic {
                sig.variadic_element.clone().unwrap_or_else(|| ty.clone())
            } else {
                ty.clone()
            });
        }
    }
    if variadic {
        // Variadic positional calls expand the declared element type; explicit VARIADIC syntax is not represented here.
        for actual in expanded_variadic {
            types.push(actual);
            declared.push(sig.variadic_element.clone()?);
        }
    }
    Some((types, declared))
}
fn resolve(
    c: &dyn CatalogView,
    info: FunctionInfo,
    args: &[Type],
    declared: &[TypeId],
) -> Selection<ResolvedFunction> {
    let Some(sig) = info.signature.as_ref() else {
        return Selection::Unknown;
    };
    let Some(result_id) = sig.return_type.as_ref() else {
        return Selection::Unknown;
    };
    let result = match resolve_polymorphic_result(c, args, declared, result_id) {
        Decision::Known(result) => result,
        Decision::Unknown => return Selection::Unknown,
    };
    Selection::Match(ResolvedFunction {
        function: info,
        result,
    })
}

/// Port of `parse_func.c:func_get_detail`, `func_match_argtypes`, and `func_select_candidate`.
pub fn select_function(
    c: &dyn CatalogView,
    schema: Option<&str>,
    name: &str,
    args: &[CallArg],
    search_path: &[String],
) -> Selection<ResolvedFunction> {
    let candidates = c.function_candidates(schema, name, search_path);
    if candidates.items.is_empty() || !candidates.complete || args.iter().any(|a| a.ty.is_none()) {
        return Selection::Unknown;
    }
    let mut viable = Vec::new();
    let mut indeterminate = false;
    for info in candidates.items {
        if info.signature.is_none() {
            return Selection::Unknown;
        }
        let Some((actual, declared)) = arrange_args(&info, args) else {
            continue;
        };
        let polymorphic = declared.iter().any(|d| {
            [
                "any",
                "anyelement",
                "anyarray",
                "anynonarray",
                "anyenum",
                "anycompatible",
                "anycompatiblearray",
                "anycompatiblenonarray",
                "anyrange",
                "anycompatiblerange",
                "anymultirange",
            ]
            .iter()
            .any(|n| poly(c, n).as_ref() == Some(d))
        });
        if polymorphic {
            match check_generic_type_consistency(c, &actual, &declared) {
                Decision::Known(true) => {}
                Decision::Known(false) => continue,
                Decision::Unknown => {
                    indeterminate = true;
                    continue;
                }
            }
        }
        let mut ok = true;
        let mut exact = 0;
        for (source, target) in actual.iter().zip(&declared) {
            if named(source) == Some(target) {
                exact += 1;
                continue;
            }
            if polymorphic
                && [
                    "any",
                    "anyelement",
                    "anyarray",
                    "anynonarray",
                    "anyenum",
                    "anycompatible",
                    "anycompatiblearray",
                    "anycompatiblenonarray",
                ]
                .iter()
                .any(|n| poly(c, n).as_ref() == Some(target))
            {
                continue;
            }
            if unknown(source) {
                continue;
            }
            match decision_coerce(c, source, target) {
                Decision::Known(true) => {}
                Decision::Known(false) => {
                    ok = false;
                    break;
                }
                Decision::Unknown => {
                    indeterminate = true;
                    ok = false;
                    break;
                }
            }
        }
        if ok {
            viable.push((info, actual, declared, exact));
        }
    }
    if indeterminate {
        return Selection::Unknown;
    }
    if viable.is_empty() {
        if args.len() == 1 && matches!(c.type_(schema, name, search_path), Lookup::Found(_)) {
            return Selection::Unknown;
        }
        return Selection::NoMatch;
    }
    let max = viable.iter().map(|v| v.3).max().unwrap_or(0);
    viable.retain(|v| v.3 == max);
    if viable.len() != 1 {
        return Selection::Unknown;
    }
    let (info, actual, declared, _) = viable.pop().unwrap();
    resolve(c, info, &actual, &declared)
}

#[cfg(all(test, feature = "db"))]
mod tests {
    use super::{select_function, select_operator};
    use crate::lookup::CatalogView;
    use crate::typing::{CallArg, Selection, Type};
    use crate::{Catalog, CatalogBase, Snapshot};
    use sqlx::PgPool;
    use std::sync::Arc;

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn selects_builtin_overloads(test_db: PgPool) {
        let snapshot = Snapshot::load(&test_db).await.expect("snapshot");
        let catalog = Catalog::new(Some(Arc::new(CatalogBase::new(Arc::new(snapshot)))));
        let path = vec!["public".to_owned()];
        let int4 = catalog
            .type_(Some("pg_catalog"), "int4", &path)
            .found()
            .unwrap()
            .id
            .unwrap();
        let text = catalog
            .type_(Some("pg_catalog"), "text", &path)
            .found()
            .unwrap()
            .id
            .unwrap();
        assert!(matches!(
            select_operator(
                &catalog,
                None,
                "+",
                crate::typing::OperatorKind::Infix,
                Some(&Type::Named(int4.clone())),
                &Type::UnknownLiteral,
                &path
            ),
            Selection::Match(_)
        ));
        assert!(matches!(
            select_function(
                &catalog,
                None,
                "length",
                &[CallArg {
                    ty: Some(Type::Named(int4)),
                    name: None
                }],
                &path
            ),
            Selection::NoMatch
        ));
        assert!(
            matches!(select_operator(&catalog, None, "=", crate::typing::OperatorKind::Infix, Some(&Type::UnknownLiteral), &Type::UnknownLiteral, &path), Selection::Match(o) if o.operator.left.as_ref() == Some(&text))
        );
    }
}
