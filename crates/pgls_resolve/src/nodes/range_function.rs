use pgls_query::{
    Node, NodeEnum,
    protobuf::{FuncCall, RangeFunction},
};
use pgls_query_ext::utils::{QualifiedName, qualified_name, string_value, string_values};

use super::{
    Resolver, alias::apply_alias, func_call::resolve_func_call, record_unresolved_node,
    resolve_node,
};
use crate::{
    expr::call_args,
    scope::{Item, Level, combine_columns},
    utils::{named, row_columns},
};
use pgls_catalog::rename;
use pgls_catalog::{
    Lookup,
    typing::{Selection, Type, TypedColumn, select_function},
};

/// A function in FROM, including `ROWS FROM (...)` and `WITH ORDINALITY`. Functions see the FROM
/// items before them. Port of [`transformRangeFunction`] and [`addRangeTableEntryForFunction`].
///
/// [`transformRangeFunction`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_clause.c#L463
/// [`addRangeTableEntryForFunction`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_relation.c#L1783
pub(super) fn resolve_range_function(
    r: &mut Resolver,
    n: &RangeFunction,
    preceding: &Level,
) -> Item {
    if let Some(expanded) = expand_unnest(n) {
        return resolve_range_function(r, &expanded, preceding);
    }
    r.enter_level(preceding.clone());

    // The names of the columns, from the return columns that all overloads agree on and the
    // column definitions.
    let mut columns: Option<Vec<String>> = Some(Vec::new());
    let mut function_name = None;
    for entry in &n.functions {
        // Each entry is a list of the function call and its column definitions.
        let Some(NodeEnum::List(entry)) = entry.node.as_ref() else {
            r.depends_on_file();
            columns = None;
            continue;
        };
        let call = entry.items.first();
        let column_definitions = entry.items.get(1).and_then(|node| match &node.node {
            Some(NodeEnum::List(list)) if !list.items.is_empty() => Some(&list.items),
            _ => None,
        });

        let mut function_columns = None;
        match call.and_then(|call| call.node.as_ref()) {
            Some(NodeEnum::FuncCall(call)) => {
                resolve_func_call(r, call);
                function_columns = return_columns(r, call);
                if n.functions.len() == 1 {
                    function_name = call
                        .funcname
                        .last()
                        .and_then(string_value)
                        .map(str::to_owned);
                }
            }
            Some(_) => {
                if let Some(call) = call {
                    resolve_node(r, call);
                }
            }
            None => {}
        }
        if let Some(definitions) = column_definitions {
            for definition in definitions {
                record_unresolved_node(r, definition, false);
            }
            function_columns = column_definition_names(definitions);
        }

        columns = match (columns, function_columns) {
            (Some(mut columns), Some(function_columns)) => {
                columns.extend(function_columns);
                Some(columns)
            }
            _ => None,
        };
    }

    r.exit_level();

    if !n.coldeflist.is_empty() {
        for definition in &n.coldeflist {
            record_unresolved_node(r, definition, false);
        }
        columns = column_definition_names(&n.coldeflist);
    }
    if n.ordinality
        && let Some(columns) = columns.as_mut()
    {
        columns.push("ordinality".into());
    }
    // The function is selected outside the FROM items before it, so a LATERAL argument has no
    // type there.
    let mut typed_columns = typed_return_columns(r, n);
    // Column definitions give the names and types of a function returning `record`.
    if !n.coldeflist.is_empty() {
        typed_columns = column_definitions(r, &n.coldeflist);
    }
    if n.ordinality
        && let Some(columns) = typed_columns.as_mut()
    {
        columns.push(TypedColumn {
            name: "ordinality".into(),
            ty: named(r, "int8"),
        });
    }

    // When the overloads disagree on their columns, the selected one decides, see
    // `combine_columns`.
    let alias = n.alias.as_ref();
    let colnames = alias.map_or(&[][..], |alias| alias.colnames.as_slice());
    let (columns, types_known) = combine_columns(
        rename(columns, colnames, |column, name| *column = name),
        apply_alias(typed_columns, alias),
    );
    Item::named(
        alias.map(|alias| alias.aliasname.clone()).or(function_name),
        columns,
        types_known,
    )
}

/// `unnest(a, b)` in FROM, also as an entry of `ROWS FROM`, is `unnest(a), unnest(b)`. Port of
/// the special case in [`transformRangeFunction`].
///
/// [`transformRangeFunction`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_clause.c#L463
fn expand_unnest(n: &RangeFunction) -> Option<RangeFunction> {
    if !n.coldeflist.is_empty() {
        return None;
    }
    let mut expanded = false;
    let mut functions = Vec::new();
    for entry in &n.functions {
        match multi_argument_unnest(entry) {
            Some(call) => {
                expanded = true;
                functions.extend(call.args.iter().map(|arg| unnest_entry(call, arg)));
            }
            None => functions.push(entry.clone()),
        }
    }
    expanded.then(|| RangeFunction {
        is_rowsfrom: true,
        functions,
        ..n.clone()
    })
}

/// The call of a `ROWS FROM` entry if it is `unnest` with several arguments and nothing else.
fn multi_argument_unnest(entry: &Node) -> Option<&FuncCall> {
    let Some(NodeEnum::List(entry)) = entry.node.as_ref() else {
        return None;
    };
    let has_column_definitions = matches!(
        entry.items.get(1).and_then(|node| node.node.as_ref()),
        Some(NodeEnum::List(list)) if !list.items.is_empty()
    );
    let Some(NodeEnum::FuncCall(call)) = entry.items.first().and_then(|call| call.node.as_ref())
    else {
        return None;
    };
    let is_unnest =
        matches!(string_values(&call.funcname).as_deref(), Some([name]) if name == "unnest");
    (is_unnest
        && !has_column_definitions
        && call.args.len() > 1
        && call.agg_order.is_empty()
        && call.agg_filter.is_none()
        && call.over.is_none()
        && !call.agg_star
        && !call.agg_distinct
        && !call.func_variadic)
        .then_some(call.as_ref())
}

/// A `ROWS FROM` entry calling `pg_catalog.unnest(arg)`.
fn unnest_entry(call: &FuncCall, arg: &Node) -> Node {
    let string = |sval: &str| Node {
        node: Some(NodeEnum::String(pgls_query::protobuf::String {
            sval: sval.to_owned(),
        })),
    };
    let unnest = FuncCall {
        funcname: vec![string("pg_catalog"), string("unnest")],
        args: vec![arg.clone()],
        location: call.location,
        ..Default::default()
    };
    Node {
        node: Some(NodeEnum::List(pgls_query::protobuf::List {
            items: vec![
                Node {
                    node: Some(NodeEnum::FuncCall(Box::new(unnest))),
                },
                Node {
                    node: Some(NodeEnum::List(pgls_query::protobuf::List::default())),
                },
            ],
        })),
    }
}

/// The output columns of a function call in FROM, if all its overloads agree on them.
fn return_columns(r: &Resolver, call: &FuncCall) -> Option<Vec<String>> {
    let QualifiedName { schema, name, .. } = qualified_name(&call.funcname)?;
    let Lookup::Found(overloads) = r.catalog.functions(schema, name, r.search_path) else {
        return None;
    };
    let first = overloads.first()?.return_columns.clone()?;
    overloads
        .iter()
        .all(|overload| overload.return_columns.as_ref() == Some(&first))
        .then(|| first.into_iter().map(|column| column.name).collect())
}

fn typed_return_columns(r: &mut Resolver, n: &RangeFunction) -> Option<Vec<TypedColumn>> {
    let mut output = Vec::new();
    for entry in &n.functions {
        let NodeEnum::List(entry) = entry.node.as_ref()? else {
            return None;
        };
        let call = entry.items.first()?.node.as_ref()?;
        let NodeEnum::FuncCall(call) = call else {
            return None;
        };
        let definitions = entry.items.get(1).and_then(|node| match &node.node {
            Some(NodeEnum::List(list)) if !list.items.is_empty() => Some(&list.items),
            _ => None,
        });
        let QualifiedName { schema, name, .. } = qualified_name(&call.funcname)?;
        if call.agg_within_group {
            return None;
        }
        let args = call_args(r, call);
        let Selection::Match(selected) = select_function(
            r.catalog,
            schema,
            name,
            &args,
            call.func_variadic,
            false,
            r.search_path,
        ) else {
            // Column definitions give the columns without the function.
            output.extend(column_definitions(r, definitions?)?);
            continue;
        };
        r.record_call(call, &selected);
        if let Some(definitions) = definitions {
            output.extend(column_definitions(r, definitions)?);
            continue;
        }
        match selected.function.return_columns {
            // A single output parameter of a composite type expands to its attributes.
            Some(columns) if columns.len() == 1 => {
                let column = columns.into_iter().next()?;
                match row_columns(r, column.ty.as_ref()?)? {
                    Some(attributes) => output.extend(attributes),
                    None => output.push(column.into()),
                }
            }
            Some(columns) => output.extend(columns.into_iter().map(TypedColumn::from)),
            None => match row_columns(r, selected.result.as_ref()?)? {
                Some(attributes) => output.extend(attributes),
                // A scalar function yields one column, named by `chooseScalarFunctionAlias`.
                None => output.push(TypedColumn {
                    name: match (&n.alias, n.functions.len()) {
                        (Some(alias), 1) => alias.aliasname.clone(),
                        _ => name.to_owned(),
                    },
                    ty: selected.result,
                }),
            },
        }
    }
    Some(output)
}

/// The columns of a column definition list, like `(a int, b text)` in
/// `f() as t(a int, b text)`, with their types ([`addRangeTableEntryForFunction`]).
///
/// [`addRangeTableEntryForFunction`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_relation.c#L1783
fn column_definitions(r: &Resolver, definitions: &[Node]) -> Option<Vec<TypedColumn>> {
    definitions
        .iter()
        .map(|definition| match &definition.node {
            Some(NodeEnum::ColumnDef(definition)) => Some(TypedColumn {
                name: definition.colname.clone(),
                ty: definition.type_name.as_ref().and_then(|ty| {
                    let (id, _) =
                        pgls_catalog::typing::normalize_type_name(r.catalog, ty, r.search_path);
                    id.map(Type::Named)
                }),
            }),
            _ => None,
        })
        .collect()
}

fn column_definition_names(definitions: &[Node]) -> Option<Vec<String>> {
    definitions
        .iter()
        .map(|definition| match &definition.node {
            Some(NodeEnum::ColumnDef(column)) => Some(column.colname.clone()),
            _ => None,
        })
        .collect()
}
