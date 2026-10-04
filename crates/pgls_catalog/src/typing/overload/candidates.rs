//! Which overloads a call can see, with their argument types expanded for the call. Port of
//! [`FuncnameGetCandidates`] and [`OpernameGetCandidates`].
//!
//! [`FuncnameGetCandidates`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/catalog/namespace.c#L1192
//! [`OpernameGetCandidates`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/catalog/namespace.c#L1888

use super::Candidate;
use crate::typing::TypeId;
use crate::{FunctionInfo, OperatorInfo, OperatorKind};

/// Port of [`FuncnameGetCandidates`] for a call with `nargs` arguments, the last `names.len()`
/// of which are named: expands variadic and default arguments, and hides functions with the same
/// arguments later in the search path. Returns `None` when a candidate can't be evaluated.
///
/// [`FuncnameGetCandidates`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/catalog/namespace.c#L1192
pub(super) fn expand_candidates(
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
/// parameters filled in from defaults. Port of [`MatchNamedCall`].
///
/// [`MatchNamedCall`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/catalog/namespace.c#L1585
pub(super) fn match_named_call(
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

/// Port of [`OpernameGetCandidates`]: an operator hides operators with the same
/// argument types later in the search path. `operators` come in search path order. `None` when
/// an operator's argument types are unknown.
///
/// [`OpernameGetCandidates`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/catalog/namespace.c#L1888
pub(super) fn visible_operators(
    operators: Vec<OperatorInfo>,
    kind: OperatorKind,
) -> Option<Vec<Candidate<OperatorInfo>>> {
    let mut visible: Vec<Candidate<OperatorInfo>> = Vec::new();
    for operator in operators {
        let args = match kind {
            OperatorKind::Infix => vec![operator.left.clone(), operator.right.clone()],
            OperatorKind::Prefix => vec![operator.right.clone()],
        };
        let args = args.into_iter().collect::<Option<Vec<_>>>()?;
        if visible.iter().any(|seen| seen.args == args) {
            continue;
        }
        visible.push(Candidate {
            item: operator,
            args,
            ambiguous: false,
        });
    }
    Some(visible)
}
