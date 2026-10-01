//! Operator and function overload resolution, ported from Postgres' `parse_func.c`,
//! `parse_oper.c`, and `namespace.c`.
//!
//! Selection only proves an error (`NoMatch` or `Ambiguous`) when the candidate list is
//! complete and every step of the algorithm had the information it needed. Anything else is
//! `Unknown`.

use crate::typing::coerce::{base_type, type_category};
use crate::typing::{
    CallArg, CoercionContext, Decision, OperatorInfo, OperatorKind, ResolvedFunction,
    ResolvedOperator, Selection, Type, TypeId, can_coerce, check_generic_type_consistency,
    resolve_polymorphic_result,
};
use crate::{CatalogView, FunctionInfo, Lookup, TypeKind};

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

    // `OpernameGetCandidates`: an operator hides operators with the same argument types later
    // in the search path. Candidates come in search path order.
    let mut visible: Vec<Candidate<OperatorInfo>> = Vec::new();
    for operator in candidates.items {
        let args = match kind {
            OperatorKind::Infix => vec![operator.left.clone(), operator.right.clone()],
            OperatorKind::Prefix => vec![operator.right.clone()],
        };
        let Some(args) = args.into_iter().collect::<Option<Vec<_>>>() else {
            return Selection::Unknown;
        };
        if visible.iter().any(|seen| seen.args == args) {
            continue;
        }
        visible.push(Candidate {
            item: operator,
            args,
            ambiguous: false,
        });
    }

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
fn exact_operator(
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

fn resolve_operator(
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

/// Selects the function for a call. Returns `Unknown` when no function with this name and
/// argument count is visible; that's the job of the name and arity check.
///
/// Port of `parse_func.c: func_get_detail`.
pub fn select_function(
    c: &dyn CatalogView,
    schema: Option<&str>,
    name: &str,
    args: &[CallArg],
    search_path: &[String],
) -> Selection<ResolvedFunction> {
    let Some(inputs) = args
        .iter()
        .map(|arg| arg.ty.clone())
        .collect::<Option<Vec<Type>>>()
    else {
        return Selection::Unknown;
    };
    if inputs.iter().any(|input| matches!(input, Type::Record(_))) {
        return Selection::Unknown;
    }
    // Named arguments must follow the positional ones; anything else is a syntax error.
    let positional = args.iter().take_while(|arg| arg.name.is_none()).count();
    let Some(names) = args[positional..]
        .iter()
        .map(|arg| arg.name.clone())
        .collect::<Option<Vec<String>>>()
    else {
        return Selection::Unknown;
    };

    let candidates = c.function_candidates(schema, name, search_path);
    if !candidates.complete {
        return Selection::Unknown;
    }
    let Decision::Known(pseudo) = Pseudo::load(c) else {
        return Selection::Unknown;
    };
    let Some(expanded) = expand_candidates(candidates.items, inputs.len(), &names) else {
        return Selection::Unknown;
    };
    if expanded.is_empty() {
        return Selection::Unknown;
    }

    // There can be only one exact match.
    let exact = expanded.iter().find(|candidate| {
        inputs
            .iter()
            .zip(&candidate.args)
            .all(|(input, declared)| matches!(input, Type::Named(id) if id == declared))
    });
    let best = if let Some(exact) = exact {
        Some(exact.clone())
    } else {
        // A one-argument call of a type name may be a cast, like `int4(x)` or `text(x)`.
        if inputs.len() == 1
            && names.is_empty()
            && !matches!(c.type_(schema, name, search_path), Lookup::Missing)
        {
            return Selection::Unknown;
        }
        let matches = match match_argtypes(c, &pseudo, &inputs, expanded) {
            Decision::Known(matches) => matches,
            Decision::Unknown => return Selection::Unknown,
        };
        match matches.len() {
            0 => {
                // `ParseFuncOrColumn` tries `col(row)` as a field selection next.
                if inputs.len() == 1 && names.is_empty() && !is_scalar(c, &inputs[0]) {
                    return Selection::Unknown;
                }
                return Selection::NoMatch;
            }
            1 => matches.into_iter().next(),
            _ => match select_candidate(c, &pseudo, &inputs, matches) {
                Decision::Known(best) => best,
                Decision::Unknown => return Selection::Unknown,
            },
        }
    };
    let Some(best) = best else {
        return Selection::Ambiguous;
    };
    if best.ambiguous {
        return Selection::Ambiguous;
    }

    let Some(result) = best
        .item
        .signature
        .as_ref()
        .and_then(|signature| signature.return_type.clone())
    else {
        return Selection::Unknown;
    };
    let result = match resolve_result(c, &pseudo, &inputs, &best.args, &result) {
        Decision::Known(result) => result,
        Decision::Unknown => None,
    };
    Selection::Match(ResolvedFunction {
        function: best.item,
        result,
    })
}

/// Whether a value of this type is certainly not a row, so `col(value)` can't be a field
/// selection.
fn is_scalar(c: &dyn CatalogView, ty: &Type) -> bool {
    let Type::Named(id) = ty else {
        return matches!(ty, Type::UnknownLiteral);
    };
    match c.type_by_id(id) {
        Lookup::Found(info) => matches!(
            info.kind,
            Some(TypeKind::Base | TypeKind::Enum | TypeKind::Range | TypeKind::Multirange)
        ),
        _ => false,
    }
}

/// An operator or function with its declared argument types, in the order of the call's
/// arguments.
#[derive(Clone)]
struct Candidate<T> {
    item: T,
    args: Vec<TypeId>,
    /// Another function in the same schema matches the call the same way, so choosing this
    /// candidate is ambiguous.
    ambiguous: bool,
}

/// Port of `namespace.c: FuncnameGetCandidates` for a call with `nargs` arguments, the last
/// `names.len()` of which are named. Returns `None` when a candidate can't be evaluated.
fn expand_candidates(
    functions: Vec<FunctionInfo>,
    nargs: usize,
    names: &[String],
) -> Option<Vec<Candidate<FunctionInfo>>> {
    // Candidates, and whether each expands variadic arguments.
    let mut expanded: Vec<(Candidate<FunctionInfo>, bool)> = Vec::new();
    for function in functions {
        let signature = function.signature.as_ref()?;
        let params = signature
            .arguments
            .iter()
            .map(|arg| arg.ty.clone())
            .collect::<Option<Vec<_>>>()?;
        let pronargs = params.len();
        let defaults = signature.input_defaults;
        let use_defaults = pronargs > nargs;
        if use_defaults && nargs + defaults < pronargs {
            continue;
        }

        let mut variadic = false;
        let full: Vec<TypeId> = if names.is_empty() {
            variadic = pronargs <= nargs && signature.variadic_element.is_some();
            if pronargs != nargs && !variadic && !use_defaults {
                continue;
            }
            if variadic {
                let element = signature.variadic_element.clone()?;
                let mut full = params[..pronargs - 1].to_vec();
                full.resize(nargs, element);
                full
            } else {
                params.clone()
            }
        } else {
            // Named arguments can't match the expanded variadic arguments.
            if signature.variadic_element.is_some() {
                continue;
            }
            if pronargs != nargs && !use_defaults {
                continue;
            }
            let names_of: Vec<Option<&str>> = signature
                .arguments
                .iter()
                .map(|arg| arg.name.as_deref())
                .collect();
            let Some(order) = match_named_call(&names_of, nargs, names, defaults) else {
                continue;
            };
            order
                .iter()
                .map(|&position| params[position].clone())
                .collect()
        };

        let candidate = Candidate {
            item: function,
            args: full[..nargs].to_vec(),
            ambiguous: false,
        };
        // Candidates with the same types for the call's arguments: one earlier in the search
        // path hides the later one. In the same schema, a non-variadic candidate beats one that
        // expands variadic arguments; otherwise the call is ambiguous.
        if let Some(index) = expanded
            .iter()
            .position(|(previous, _)| previous.args == candidate.args)
        {
            let (previous, previous_variadic) = &mut expanded[index];
            if previous.item.schema != candidate.item.schema || (variadic && !*previous_variadic) {
                continue;
            }
            if !variadic && *previous_variadic {
                expanded.remove(index);
            } else {
                previous.ambiguous = true;
                continue;
            }
        }
        expanded.push((candidate, variadic));
    }
    Some(
        expanded
            .into_iter()
            .map(|(candidate, _)| candidate)
            .collect(),
    )
}

/// Maps the call's arguments to parameter positions, followed by the positions of the
/// parameters filled in from defaults. Port of `namespace.c: MatchNamedCall`.
fn match_named_call(
    params: &[Option<&str>],
    nargs: usize,
    names: &[String],
    defaults: usize,
) -> Option<Vec<usize>> {
    let pronargs = params.len();
    if params.iter().all(Option::is_none) {
        return None;
    }
    let positional = nargs - names.len();
    if positional > pronargs {
        return None;
    }
    let mut given = vec![false; pronargs];
    let mut order: Vec<usize> = (0..positional).collect();
    given[..positional].fill(true);
    for name in names {
        let position = params
            .iter()
            .position(|param| *param == Some(name.as_str()))?;
        if given[position] {
            return None;
        }
        given[position] = true;
        order.push(position);
    }
    let first_default = pronargs - defaults.min(pronargs);
    for (position, given) in given.iter().enumerate().skip(positional) {
        if !given {
            if position < first_default {
                return None;
            }
            order.push(position);
        }
    }
    Some(order)
}

/// Keeps the candidates the inputs can be coerced to. Port of
/// `parse_func.c: func_match_argtypes`.
fn match_argtypes<T>(
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
fn can_coerce_args(
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
fn select_candidate<T: Clone>(
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
fn keep_best<T>(candidates: &mut Vec<Candidate<T>>, mut score: impl FnMut(&Candidate<T>) -> usize) {
    let scores: Vec<usize> = candidates.iter().map(&mut score).collect();
    let best = scores.iter().copied().max().unwrap_or(0);
    let mut scores = scores.into_iter();
    candidates.retain(|_| scores.next() == Some(best));
}

/// The result type, with polymorphic result types resolved from the inputs. `None` when the
/// result is `"any"`-like or can't be resolved.
fn resolve_result(
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
struct Pseudo {
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
    use crate::lookup::{CatalogView, FunctionInfo, Lookup, RelationInfo, TypeInfo};
    use crate::typing::{CallArg, OperatorKind, Selection, Type};

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
