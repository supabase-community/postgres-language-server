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
/// items before them.
pub(super) fn resolve_range_function(
    r: &mut Resolver,
    n: &RangeFunction,
    preceding: &Level,
) -> Item {
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
        if let Some(columns) = selected.function.return_columns {
            output.extend(columns.into_iter().map(|column| TypedColumn {
                name: column.name,
                ty: column.ty,
            }));
        } else {
            // A scalar function yields one column, typed by the resolved result type.
            output.push(TypedColumn {
                name: name.to_owned(),
                ty: selected.result,
            });
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

fn column_definition_names(definitions: &[Node]) -> Columns {
    definitions
        .iter()
        .map(|definition| match &definition.node {
            Some(NodeEnum::ColumnDef(column)) => Some(column.colname.clone()),
            _ => None,
        })
        .collect()
}
