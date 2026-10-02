//! Operator and function overload resolution, ported from Postgres' `parse_func.c`,
//! `parse_oper.c`, and `namespace.c`.
//!
//! Selection only proves an error (`NoMatch` or `Ambiguous`) when the candidate list is
//! complete and every step of the algorithm had the information it needed. Anything else is
//! `Unknown`.

mod candidates;
mod function;
mod operator;

pub use function::select_function;
pub use operator::select_operator;

use crate::typing::coerce::can_coerce;
use crate::typing::types::{base_type, type_category};
use crate::typing::unify::{check_generic_type_consistency, resolve_polymorphic_result};
use crate::typing::{CoercionContext, Decision, Type, TypeId};
use crate::{CatalogView, Lookup};

/// An operator or function with its declared argument types, in the order of the call's
/// arguments.
#[derive(Clone)]
pub(super) struct Candidate<T> {
    item: T,
    args: Vec<TypeId>,
    /// Another function in the same schema matches the call the same way, so choosing this
    /// candidate is ambiguous.
    ambiguous: bool,
}

/// Keeps the candidates the inputs can be coerced to. Port of
/// `parse_func.c: func_match_argtypes`.
pub(super) fn match_argtypes<T>(
    c: &dyn CatalogView,
    pseudo: &Pseudo,
    inputs: &[Type],
    candidates: Vec<Candidate<T>>,
) -> Decision<Vec<Candidate<T>>> {
    let mut matches = Vec::new();
    for candidate in candidates {
        match can_coerce_args(c, pseudo, inputs, &candidate.args) {
            Decision::Known(true) => matches.push(candidate),
            Decision::Known(false) => {}
            Decision::Unknown => return Decision::Unknown,
        }
    }
    Decision::Known(matches)
}

/// Whether all inputs can be coerced implicitly to the targets, including the consistency
/// of polymorphic arguments. Port of `parse_coerce.c: can_coerce_type`.
pub(super) fn can_coerce_args(
    c: &dyn CatalogView,
    pseudo: &Pseudo,
    inputs: &[Type],
    targets: &[TypeId],
) -> Decision<bool> {
    let mut generic = false;
    let mut unknown = false;
    for (input, target) in inputs.iter().zip(targets) {
        if matches!(input, Type::Named(id) if id == target) || pseudo.any.as_ref() == Some(target) {
            continue;
        }
        if pseudo.polymorphic.contains(target) {
            generic = true;
            continue;
        }
        if matches!(input, Type::UnknownLiteral) {
            continue;
        }
        match can_coerce(c, input, target, CoercionContext::Implicit) {
            Decision::Known(true) => {}
            Decision::Known(false) => return Decision::Known(false),
            Decision::Unknown => unknown = true,
        }
    }
    if generic {
        match check_generic_type_consistency(c, inputs, targets) {
            Decision::Known(true) => {}
            Decision::Known(false) => return Decision::Known(false),
            Decision::Unknown => unknown = true,
        }
    }
    if unknown {
        Decision::Unknown
    } else {
        Decision::Known(true)
    }
}

/// Picks the best of several matching candidates, or `None` if there is no unique best one.
/// Port of `parse_func.c: func_select_candidate`.
pub(super) fn select_candidate<T: Clone>(
    c: &dyn CatalogView,
    pseudo: &Pseudo,
    inputs: &[Type],
    mut candidates: Vec<Candidate<T>>,
) -> Decision<Option<Candidate<T>>> {
    // Domains are reduced to their base types. `None` is an unknown-type input.
    let mut base_inputs: Vec<Option<TypeId>> = Vec::with_capacity(inputs.len());
    for input in inputs {
        base_inputs.push(match input {
            Type::Named(id) => match base_type(c, id) {
                Decision::Known(base) => Some(base),
                Decision::Unknown => return Decision::Unknown,
            },
            Type::UnknownLiteral => None,
            Type::Record(_) => return Decision::Unknown,
        });
    }

    // Keep the candidates with the most exact matches on the known inputs.
    let exact_matches = |candidate: &Candidate<T>| {
        candidate
            .args
            .iter()
            .zip(&base_inputs)
            .filter(|(declared, input)| input.as_ref() == Some(*declared))
            .count()
    };
    keep_best(&mut candidates, exact_matches);
    if candidates.len() == 1 {
        return Decision::Known(candidates.pop());
    }

    // Then the most exact matches or preferred types of the input's category at the
    // positions that need a coercion.
    let mut input_categories = Vec::with_capacity(base_inputs.len());
    for input in &base_inputs {
        input_categories.push(match input {
            Some(id) => match type_category(c, id) {
                Decision::Known((category, _)) => Some(category),
                Decision::Unknown => return Decision::Unknown,
            },
            None => None,
        });
    }
    let mut scores = Vec::with_capacity(candidates.len());
    for candidate in &candidates {
        let mut score = 0;
        for ((declared, input), category) in candidate
            .args
            .iter()
            .zip(&base_inputs)
            .zip(&input_categories)
        {
            let Some(input) = input else { continue };
            if declared == input {
                score += 1;
                continue;
            }
            match type_category(c, declared) {
                Decision::Known((declared_category, preferred)) => {
                    if preferred && Some(declared_category) == *category {
                        score += 1;
                    }
                }
                Decision::Unknown => return Decision::Unknown,
            }
        }
        scores.push(score);
    }
    let mut scores = scores.into_iter();
    keep_best(&mut candidates, |_| scores.next().unwrap_or(0));
    if candidates.len() == 1 {
        return Decision::Known(candidates.pop());
    }

    // Without unknown-type inputs, there are no more heuristics.
    let unknowns = base_inputs.iter().filter(|input| input.is_none()).count();
    if unknowns == 0 {
        return Decision::Known(None);
    }

    // Resolve the category of each unknown-type input. STRING wins if any candidate accepts
    // it; otherwise all candidates must agree.
    let mut slots: Vec<Option<(char, bool)>> = vec![None; inputs.len()];
    let mut resolved = true;
    'positions: for position in 0..inputs.len() {
        if base_inputs[position].is_some() {
            continue;
        }
        let mut slot: Option<(char, bool)> = None;
        let mut conflict = false;
        for candidate in &candidates {
            let (category, preferred) = match type_category(c, &candidate.args[position]) {
                Decision::Known(category) => category,
                Decision::Unknown => return Decision::Unknown,
            };
            slot = match slot {
                None => Some((category, preferred)),
                Some((current, has_preferred)) if current == category => {
                    Some((current, has_preferred || preferred))
                }
                Some(_) if category == 'S' => Some((category, preferred)),
                Some(current) => {
                    conflict = true;
                    Some(current)
                }
            };
        }
        if conflict && slot.map(|(category, _)| category) != Some('S') {
            resolved = false;
            break 'positions;
        }
        slots[position] = slot;
    }
    if resolved {
        let mut kept = Vec::new();
        for candidate in &candidates {
            let mut keep = true;
            for (position, slot) in slots.iter().enumerate() {
                let Some((category, has_preferred)) = slot else {
                    continue;
                };
                let (declared_category, preferred) =
                    match type_category(c, &candidate.args[position]) {
                        Decision::Known(category) => category,
                        Decision::Unknown => return Decision::Unknown,
                    };
                if declared_category != *category || (*has_preferred && !preferred) {
                    keep = false;
                    break;
                }
            }
            if keep {
                kept.push(candidate.clone());
            }
        }
        if !kept.is_empty() {
            candidates = kept;
        }
        if candidates.len() == 1 {
            return Decision::Known(candidates.pop());
        }
    }

    // Last gasp: if all known inputs have the same type, assume the unknown inputs have it
    // too, and look for a unique candidate that accepts that.
    if unknowns < inputs.len() {
        let mut known = base_inputs.iter().flatten();
        let first = known.next().cloned();
        if let Some(first) = first.filter(|first| known.all(|other| other == first)) {
            let assumed = vec![Type::Named(first); inputs.len()];
            let mut unique = None;
            for candidate in &candidates {
                match can_coerce_args(c, pseudo, &assumed, &candidate.args) {
                    Decision::Known(true) => {
                        if unique.is_some() {
                            return Decision::Known(None);
                        }
                        unique = Some(candidate.clone());
                    }
                    Decision::Known(false) => {}
                    Decision::Unknown => return Decision::Unknown,
                }
            }
            if unique.is_some() {
                return Decision::Known(unique);
            }
        }
    }
    Decision::Known(None)
}

/// Keeps the candidates with the highest score, or all of them if every score is zero.
pub(super) fn keep_best<T>(
    candidates: &mut Vec<Candidate<T>>,
    mut score: impl FnMut(&Candidate<T>) -> usize,
) {
    let scores: Vec<usize> = candidates.iter().map(&mut score).collect();
    let best = scores.iter().copied().max().unwrap_or(0);
    let mut scores = scores.into_iter();
    candidates.retain(|_| scores.next() == Some(best));
}

/// The result type, with polymorphic result types resolved from the inputs. `None` when the
/// result is `"any"`-like or can't be resolved.
pub(super) fn resolve_result(
    c: &dyn CatalogView,
    pseudo: &Pseudo,
    inputs: &[Type],
    declared: &[TypeId],
    result: &TypeId,
) -> Decision<Option<Type>> {
    if !pseudo.polymorphic.contains(result)
        && !declared.iter().any(|d| pseudo.polymorphic.contains(d))
    {
        return Decision::Known(Some(Type::Named(result.clone())));
    }
    resolve_polymorphic_result(c, inputs, declared, result)
}

/// The identities of `"any"` and the polymorphic pseudo-types.
pub(super) struct Pseudo {
    any: Option<TypeId>,
    polymorphic: Vec<TypeId>,
}

impl Pseudo {
    fn load(c: &dyn CatalogView) -> Decision<Self> {
        let lookup = |name: &str| match c.type_(Some("pg_catalog"), name, &[]) {
            Lookup::Found(info) => Decision::Known(info.id),
            Lookup::Missing => Decision::Known(None),
            Lookup::Unknown => Decision::Unknown,
        };
        let Decision::Known(any) = lookup("any") else {
            return Decision::Unknown;
        };
        let mut polymorphic = Vec::new();
        for name in [
            "anyelement",
            "anyarray",
            "anynonarray",
            "anyenum",
            "anyrange",
            "anymultirange",
            "anycompatible",
            "anycompatiblearray",
            "anycompatiblenonarray",
            "anycompatiblerange",
            "anycompatiblemultirange",
        ] {
            match lookup(name) {
                Decision::Known(Some(id)) => polymorphic.push(id),
                Decision::Known(None) => {}
                Decision::Unknown => return Decision::Unknown,
            }
        }
        Decision::Known(Self { any, polymorphic })
    }
}

#[cfg(test)]
mod incomplete_candidates_tests {
    use super::{select_function, select_operator};
    use crate::OperatorKind;
    use crate::lookup::{CatalogView, FunctionInfo, Lookup, RelationInfo, TypeInfo};
    use crate::typing::{CallArg, Selection, Type};

    struct Incomplete;

    impl CatalogView for Incomplete {
        fn relation(&self, _: Option<&str>, _: &str, _: &[String]) -> Lookup<RelationInfo> {
            Lookup::Unknown
        }
        fn schema(&self, _: &str) -> Lookup<()> {
            Lookup::Unknown
        }
        fn functions(&self, _: Option<&str>, _: &str, _: &[String]) -> Lookup<Vec<FunctionInfo>> {
            Lookup::Unknown
        }
        fn type_(&self, _: Option<&str>, _: &str, _: &[String]) -> Lookup<TypeInfo> {
            Lookup::Unknown
        }
    }

    #[test]
    fn incomplete_candidate_lists_never_prove_errors() {
        let catalog = Incomplete;
        let args = [CallArg {
            ty: Some(Type::UnknownLiteral),
            name: None,
        }];
        assert!(matches!(
            select_function(&catalog, None, "f", &args, &[]),
            Selection::Unknown
        ));
        assert!(matches!(
            select_operator(
                &catalog,
                None,
                "+",
                OperatorKind::Infix,
                Some(&Type::UnknownLiteral),
                &Type::UnknownLiteral,
                &[]
            ),
            Selection::Unknown
        ));
    }
}
