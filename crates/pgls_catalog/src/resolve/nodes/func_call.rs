use pgls_query::{
    NodeEnum,
    protobuf::{CoercionForm, FuncCall},
};

use super::{
    Resolver, resolve_list, resolve_node,
    string::{string_value, string_values},
};
use crate::lookup::Lookup;
use crate::resolve::{FindingKind, scope::find_item};

pub(super) fn resolve_func_call(r: &mut Resolver, n: &FuncCall) {
    resolve_list(r, &n.args);
    resolve_list(r, &n.agg_order);
    if let Some(filter) = n.agg_filter.as_deref() {
        resolve_node(r, filter);
    }
    if let Some(window) = n.over.as_deref() {
        resolve_list(r, &window.partition_clause);
        resolve_list(r, &window.order_clause);
    }

    // SQL syntax like EXTRACT, TRIM, or AT TIME ZONE calls functions in pg_catalog.
    if n.funcformat() == CoercionForm::CoerceSqlSyntax {
        return;
    }

    let Some(names) = string_values(&n.funcname) else {
        r.depends_on_file();
        return;
    };
    let (schema, name) = match names.as_slice() {
        [name] => (None, name.as_str()),
        [schema, name] => (Some(schema.as_str()), name.as_str()),
        _ => {
            r.depends_on_file();
            return;
        }
    };
    if let Some(schema) = schema
        && r.check_schema(schema, n.location)
    {
        return;
    }

    let arg_count = if n.agg_star { 0 } else { n.args.len() };
    let unknown_function = |name_exists| FindingKind::UnknownFunction {
        schema: schema.map(str::to_owned),
        name: name.to_owned(),
        arg_count,
        name_exists,
    };
    match r.catalog.functions(schema, name, r.search_path) {
        Lookup::Found(overloads) => {
            for overload in &overloads {
                r.uses(overload.origin);
            }
            // Ordered-set aggregates count their `WITHIN GROUP` arguments, and `VARIADIC`
            // passes an array for any number of arguments.
            if n.func_variadic || n.agg_within_group {
                return;
            }
            let accepts = overloads.iter().any(|overload| {
                overload.min_args <= arg_count
                    && overload.max_args.is_none_or(|max| arg_count <= max)
            });
            if !accepts && !may_be_field_access(r, n) && !may_be_type_coercion(r, schema, name, n) {
                r.report(unknown_function(true), n.location);
            }
        }
        Lookup::Missing => {
            if !may_be_field_access(r, n) && !may_be_type_coercion(r, schema, name, n) {
                r.report(unknown_function(false), n.location);
            }
        }
        Lookup::Unknown => r.depends_on_file(),
    }
}

/// Postgres reads a one-argument call of a type name as a cast when no function matches:
/// `inet(x)` is `x::inet`. Port of the coercion fallback in [`func_get_detail`], which finds
/// the type with [`FuncNameAsType`].
///
/// [`func_get_detail`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_func.c#L1450
/// [`FuncNameAsType`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_func.c#L1936
fn may_be_type_coercion(r: &Resolver, schema: Option<&str>, name: &str, n: &FuncCall) -> bool {
    let [argument] = n.args.as_slice() else {
        return false;
    };
    if n.agg_star || matches!(argument.node, Some(NodeEnum::NamedArgExpr(_))) {
        return false;
    }
    !matches!(
        r.catalog.type_(schema, name, r.search_path),
        Lookup::Missing
    )
}

/// Postgres reads `name(row)` as a field access if `row` is a whole row: `name(t)` is the same
/// as `t.name` ([`ParseComplexProjection`], from [`ParseFuncOrColumn`]).
///
/// [`ParseComplexProjection`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_func.c#L1967
/// [`ParseFuncOrColumn`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_func.c#L90
fn may_be_field_access(r: &Resolver, n: &FuncCall) -> bool {
    let [argument] = n.args.as_slice() else {
        return false;
    };
    match argument.node.as_ref() {
        Some(NodeEnum::ColumnRef(column)) => match column.fields.as_slice() {
            [name] => string_value(name).is_some_and(|name| {
                find_item(&r.levels, name).is_some() || r.function_param(name).is_some()
            }),
            _ => false,
        },
        Some(NodeEnum::ParamRef(_) | NodeEnum::RowExpr(_) | NodeEnum::AIndirection(_)) => true,
        _ => false,
    }
}
