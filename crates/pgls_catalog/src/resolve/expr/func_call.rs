use super::*;

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
        let names = string_values(&n.funcname)?;
        let (schema, name) = match names.as_slice() {
            [name] => (None, name.as_str()),
            [schema, name] => (Some(schema.as_str()), name.as_str()),
            _ => return None,
        };
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
                r.record_call(n, &f.function);
            }
            return None;
        }
        let failure = match selection {
            Selection::Match(f) => {
                r.record_call(n, &f.function);
                return f.result;
            }
            Selection::NoMatch => crate::resolve::MatchFailure::NoMatch,
            Selection::Ambiguous => crate::resolve::MatchFailure::Ambiguous,
            Selection::Unknown => return None,
        };
        let span = r.name_span(n.location);
        r.report_with_span(
            crate::resolve::FindingKind::FunctionArgumentMismatch {
                schema: schema.map(str::to_owned),
                name: name.to_owned(),
                args: args.into_iter().filter_map(|a| a.ty).collect(),
                failure,
            },
            span,
        );
        None
    }
}

/// The arguments of a call with their inferred types and names.
pub(in crate::resolve) fn call_args(
    r: &mut Resolver<'_>,
    n: &pgls_query::protobuf::FuncCall,
) -> Vec<CallArg> {
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
