//! Operator selection. Port of `parse_oper.c`.

use super::candidates::visible_operators;
use super::{Candidate, Pseudo, match_argtypes, resolve_result, select_candidate};
use crate::typing::types::base_type;
use crate::typing::{Decision, ResolvedOperator, Selection, Type, TypeId};
use crate::{CatalogView, OperatorInfo, OperatorKind};

/// Selects the operator for an operator expression. `left` is `None` for prefix operators.
///
/// Port of `parse_oper.c: oper` and `left_oper`.
pub fn select_operator(
    c: &dyn CatalogView,
    schema: Option<&str>,
    name: &str,
    kind: OperatorKind,
    left: Option<&Type>,
    right: &Type,
    search_path: &[String],
) -> Selection<ResolvedOperator> {
    let inputs: Vec<Type> = match kind {
        OperatorKind::Infix => {
            let Some(left) = left else {
                return Selection::Unknown;
            };
            vec![left.clone(), right.clone()]
        }
        OperatorKind::Prefix => vec![right.clone()],
    };
    if inputs.iter().any(|input| matches!(input, Type::Record(_))) {
        return Selection::Unknown;
    }
    let candidates = c.operator_candidates(schema, name, kind, search_path);
    if !candidates.complete {
        return Selection::Unknown;
    }
    let Decision::Known(pseudo) = Pseudo::load(c) else {
        return Selection::Unknown;
    };

    let Some(visible) = visible_operators(candidates.items, kind) else {
        return Selection::Unknown;
    };

    if let Some(exact) = exact_operator(c, &visible, &inputs) {
        return match exact {
            Decision::Known(candidate) => resolve_operator(c, &pseudo, candidate, &inputs),
            Decision::Unknown => Selection::Unknown,
        };
    }

    // `oper_select_candidate`
    let matches = match match_argtypes(c, &pseudo, &inputs, visible) {
        Decision::Known(matches) => matches,
        Decision::Unknown => return Selection::Unknown,
    };
    let best = match matches.len() {
        0 => return Selection::NoMatch,
        1 => matches.into_iter().next(),
        _ => match select_candidate(c, &pseudo, &inputs, matches) {
            Decision::Known(best) => best,
            Decision::Unknown => return Selection::Unknown,
        },
    };
    match best {
        Some(candidate) => resolve_operator(c, &pseudo, candidate, &inputs),
        None => Selection::Ambiguous,
    }
}

/// Port of `parse_oper.c: binary_oper_exact` (and the exact lookup of `left_oper`). Returns
/// `None` when there is no exact match.
pub(super) fn exact_operator(
    c: &dyn CatalogView,
    visible: &[Candidate<OperatorInfo>],
    inputs: &[Type],
) -> Option<Decision<Candidate<OperatorInfo>>> {
    let find = |args: &[&TypeId]| {
        visible
            .iter()
            .find(|candidate| candidate.args.iter().eq(args.iter().copied()))
            .cloned()
    };
    match inputs {
        [Type::Named(right)] => find(&[right]).map(Decision::Known),
        [left, right] => {
            // An unknown-type side is assumed to have the other side's type.
            let (left, right, was_unknown) = match (left, right) {
                (Type::UnknownLiteral, Type::Named(right)) => (right, right, true),
                (Type::Named(left), Type::UnknownLiteral) => (left, left, true),
                (Type::Named(left), Type::Named(right)) => (left, right, false),
                _ => return None,
            };
            if let Some(found) = find(&[left, right]) {
                return Some(Decision::Known(found));
            }
            if was_unknown {
                let base = match base_type(c, left) {
                    Decision::Known(base) => base,
                    Decision::Unknown => return Some(Decision::Unknown),
                };
                if &base != left {
                    return find(&[&base, &base]).map(Decision::Known);
                }
            }
            None
        }
        _ => None,
    }
}

pub(super) fn resolve_operator(
    c: &dyn CatalogView,
    pseudo: &Pseudo,
    candidate: Candidate<OperatorInfo>,
    inputs: &[Type],
) -> Selection<ResolvedOperator> {
    let Some(result) = candidate.item.result.clone() else {
        return Selection::Unknown;
    };
    let result = match resolve_result(c, pseudo, inputs, &candidate.args, &result) {
        Decision::Known(result) => result,
        Decision::Unknown => None,
    };
    Selection::Match(ResolvedOperator {
        operator: candidate.item,
        result,
    })
}
