use pgls_query::{
    Node, NodeEnum,
    protobuf::{FuncCall, RangeFunction},
};

use super::{
    Resolver,
    alias::apply_alias,
    func_call::resolve_func_call,
    resolve_node,
    string::{string_value, string_values},
};
use crate::resolve::scope::{Columns, Item, Level};
use crate::{
    lookup::Lookup,
    typing::{CallArg, Selection, Type, TypedColumn, select_function},
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

    let mut columns: Columns = Some(Vec::new());
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
        columns = column_definition_names(&n.coldeflist);
    }
    if n.ordinality {
        if let Some(columns) = columns.as_mut() {
            columns.push("ordinality".into());
        }
    }
    let mut typed_columns = typed_return_columns(r, n);
    if n.ordinality {
        if let Some(columns) = typed_columns.as_mut() {
            columns.push(TypedColumn {
                name: "ordinality".into(),
                ty: named_type(r, "int8"),
            });
        }
    }

    // When the overloads disagree on their columns, the selected one decides.
    if columns.is_none() {
        columns = typed_columns
            .as_ref()
            .map(|columns| columns.iter().map(|column| column.name.clone()).collect());
    }

    let alias = n.alias.as_ref();
    let mut item = Item::named(
        alias.map(|alias| alias.aliasname.clone()).or(function_name),
        apply_alias(columns, alias),
    );
    item.typed_columns = typed_columns.map(|mut columns| {
        if let Some(alias) = alias {
            for (column, name) in columns.iter_mut().zip(&alias.colnames) {
                if let Some(NodeEnum::String(name)) = name.node.as_ref() {
                    column.name = name.sval.clone();
                }
            }
        }
        columns
    });
    item
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
fn return_columns(r: &Resolver, call: &FuncCall) -> Columns {
    let names = string_values(&call.funcname)?;
    let (name, schema) = match names.as_slice() {
        [name] => (name, None),
        [schema, name] => (name, Some(schema.as_str())),
        _ => return None,
    };
    let Lookup::Found(overloads) = r.catalog.functions(schema, name, r.search_path) else {
        return None;
    };
    let first = overloads.first()?.return_columns.clone()?;
    overloads
        .iter()
        .all(|overload| overload.return_columns.as_ref() == Some(&first))
        .then(|| first.into_iter().map(|column| column.name).collect())
}

fn named_type(r: &Resolver, name: &str) -> Option<Type> {
    match r.catalog.type_(Some("pg_catalog"), name, r.search_path) {
        Lookup::Found(info) => info.id.map(Type::Named),
        _ => None,
    }
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
        let name = string_values(&call.funcname)?;
        let (name, schema) = match name.as_slice() {
            [name] => (name.as_str(), None),
            [schema, name] => (name.as_str(), Some(schema.as_str())),
            _ => return None,
        };
        let args = call
            .args
            .iter()
            .map(|arg| CallArg {
                ty: arg
                    .node
                    .as_ref()
                    .and_then(|expr| super::super::expr::infer_expr(r, expr)),
                name: None,
            })
            .collect::<Vec<_>>();
        let Selection::Match(selected) =
            select_function(r.catalog, schema, name, &args, r.search_path)
        else {
            return None;
        };
        match selected.function.return_columns {
            // A single output parameter of a composite type expands to its attributes.
            Some(columns) if columns.len() == 1 => {
                let column = columns.into_iter().next()?;
                match composite_attributes(r, column.ty.as_ref()?)? {
                    Some(attributes) => output.extend(attributes),
                    None => output.push(TypedColumn {
                        name: column.name,
                        ty: column.ty,
                    }),
                }
            }
            Some(columns) => output.extend(columns.into_iter().map(|column| TypedColumn {
                name: column.name,
                ty: column.ty,
            })),
            None => match composite_attributes(r, selected.result.as_ref()?)? {
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
    if !n.coldeflist.is_empty() {
        for (column, definition) in output.iter_mut().zip(&n.coldeflist) {
            if let Some(NodeEnum::ColumnDef(definition)) = definition.node.as_ref() {
                column.name = definition.colname.clone();
                column.ty = definition.type_name.as_ref().and_then(|ty| {
                    let (id, _) = crate::typing::normalize_type_name(r.catalog, ty, r.search_path);
                    id.map(Type::Named)
                });
            }
        }
    }
    Some(output)
}

/// The attributes of a composite type, or of a domain over one, as `get_expr_result_type`
/// expands them; `Some(None)` for a scalar type and `None` when it is unknown.
fn composite_attributes(r: &Resolver, ty: &Type) -> Option<Option<Vec<TypedColumn>>> {
    let Type::Named(id) = ty else {
        return None;
    };
    let id = match crate::typing::base_type(r.catalog, id) {
        crate::typing::Decision::Known(id) => id,
        crate::typing::Decision::Unknown => return None,
    };
    let info = r.catalog.type_by_id(&id).found()?;
    match info.kind? {
        crate::TypeKind::Composite => Some(Some(
            info.attributes?
                .into_iter()
                .map(|attribute| TypedColumn {
                    name: attribute.name,
                    ty: attribute.ty,
                })
                .collect(),
        )),
        // `record` and other pseudo types need a column definition list.
        crate::TypeKind::Pseudo => None,
        _ => Some(None),
    }
}

fn column_definition_names(definitions: &[Node]) -> Columns {
    definitions
        .iter()
        .map(|definition| match &definition.node {
            Some(NodeEnum::ColumnDef(column)) => Some(column.colname.clone()),
            _ => None,
        })
        .collect()
}
