//! Function selection. Port of `parse_func.c: func_get_detail`.

use super::candidates::expand_candidates;
use super::{Pseudo, match_argtypes, resolve_result, select_candidate};
use crate::typing::{CallArg, Decision, ResolvedFunction, Selection, Type};
use crate::{CatalogView, Lookup, TypeKind};

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
pub(super) fn is_scalar(c: &dyn CatalogView, ty: &Type) -> bool {
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
