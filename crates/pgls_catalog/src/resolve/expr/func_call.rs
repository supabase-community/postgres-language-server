use super::*;

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
        // `VARIADIC` passes the variadic array itself, and ordered-set aggregates take
        // more arguments than the call lists. Neither is modelled.
        if n.func_variadic || n.agg_within_group {
            return None;
        }
        let args = n
            .args
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
            .collect::<Vec<_>>();
        let selection = select_function(r.catalog, schema, name, &args, r.search_path);
        let failure = match selection {
            Selection::Match(f) => return f.result,
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
