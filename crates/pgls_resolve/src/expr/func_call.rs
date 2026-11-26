use pgls_query::NodeEnum;
use pgls_query_ext::utils::{QualifiedName, qualified_name};

use super::infer_expr;
use crate::{resolver::Resolver, utils::named};
use pgls_catalog::typing::{CallArg, Selection, Type, select_function};

/// The result type of a function call, and a finding when no function or no unique best
/// function matches its arguments. Port of [`ParseFuncOrColumn`], which selects the function
/// with [`func_get_detail`].
///
/// [`ParseFuncOrColumn`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_func.c#L90
/// [`func_get_detail`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_func.c#L1450
pub(super) fn infer_func_call(
    r: &mut Resolver<'_>,
    n: &pgls_query::protobuf::FuncCall,
) -> Option<Type> {
    {
        let QualifiedName { schema, name, .. } = qualified_name(&n.funcname)?;
        if n.agg_star {
            return named(r, "int8");
        }
        // Ordered-set aggregates take more arguments than the call lists. Not modelled.
        if n.agg_within_group {
            return None;
        }
        let args = call_args(r, n);
        let selection = select_function(
            r.catalog,
            schema,
            name,
            &args,
            n.func_variadic,
            false,
            r.search_path,
        );
        // `VARIADIC` passes the variadic array itself. The result of such a call is not
        // modelled, and it is never reported.
        if n.func_variadic {
            if let Selection::Match(f) = &selection
                && f.function
                    .signature
                    .as_ref()
                    .is_some_and(|signature| signature.variadic_element.is_some())
            {
                r.record_call(n, f);
            }
            return None;
        }
        let failure = match selection {
            Selection::Match(f) => {
                r.record_call(n, &f);
                return f.result;
            }
            Selection::NoMatch => crate::MatchFailure::NoMatch,
            Selection::Ambiguous => crate::MatchFailure::Ambiguous,
            Selection::Unknown => return None,
        };
        let span = r.name_span(n.location);
        r.report_with_span(
            crate::FindingKind::FunctionArgumentMismatch {
                schema: schema.map(str::to_owned),
                name: name.to_owned(),
                args: args.into_iter().filter_map(|a| a.ty).collect(),
                failure,
                procedure: false,
            },
            span,
        );
        None
    }
}

/// Checks the procedure that `CALL` calls, and records it when references are being collected.
/// Port of the checks of [`ParseFuncOrColumn`] for [`transformCallStmt`]: no procedure, or no
/// unique best one, matches the arguments, or the best match is not a procedure. The findings of
/// the arguments are dropped. `CALL` passes an argument for the output parameters of a
/// procedure too, usually `NULL`, which the signatures of procedures include. Functions with the
/// same name are candidates too, with their output arguments.
///
/// [`ParseFuncOrColumn`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_func.c#L279
/// [`transformCallStmt`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/analyze.c#L3253
pub(crate) fn resolve_procedure_call(r: &mut Resolver<'_>, n: &pgls_query::protobuf::FuncCall) {
    r.depends_on_file();
    let args = r.without_findings(|r| call_args(r, n));
    let Some(QualifiedName { schema, name, .. }) = qualified_name(&n.funcname) else {
        return;
    };
    let selection = select_function(
        r.catalog,
        schema,
        name,
        &args,
        n.func_variadic,
        true,
        r.search_path,
    );
    let failure = match selection {
        Selection::Match(f) if f.function.kind == pgls_catalog::FunctionKind::Procedure => {
            r.record_call(n, &f);
            return;
        }
        // As in `infer_func_call`, a call with `VARIADIC` is never reported.
        _ if n.func_variadic => return,
        // A function, aggregate or window function.
        Selection::Match(_) => None,
        Selection::NoMatch => Some(crate::MatchFailure::NoMatch),
        Selection::Ambiguous => Some(crate::MatchFailure::Ambiguous),
        Selection::Unknown => return,
    };
    let schema = schema.map(str::to_owned);
    let name = name.to_owned();
    let args = args.into_iter().filter_map(|a| a.ty).collect();
    let kind = match failure {
        Some(failure) => crate::FindingKind::FunctionArgumentMismatch {
            schema,
            name,
            args,
            failure,
            procedure: true,
        },
        None => crate::FindingKind::NotAProcedure { schema, name, args },
    };
    r.report(kind, n.location);
}

/// The arguments of a call with their inferred types and names.
pub(crate) fn call_args(r: &mut Resolver<'_>, n: &pgls_query::protobuf::FuncCall) -> Vec<CallArg> {
    n.args
        .iter()
        .filter_map(|arg| arg.node.as_ref())
        .map(|arg| match arg {
            NodeEnum::NamedArgExpr(named) => CallArg {
                ty: named
                    .arg
                    .as_deref()
                    .and_then(|x| x.node.as_ref())
                    .and_then(|x| infer_expr(r, x)),
                name: Some(named.name.clone()),
            },
            arg => CallArg {
                ty: infer_expr(r, arg),
                name: None,
            },
        })
        .collect()
}
