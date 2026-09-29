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
use crate::view::Lookup;

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

    let alias = n.alias.as_ref();
    Item::named(
        alias.map(|alias| alias.aliasname.clone()).or(function_name),
        apply_alias(columns, alias),
    )
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

fn column_definition_names(definitions: &[Node]) -> Columns {
    definitions
        .iter()
        .map(|definition| match &definition.node {
            Some(NodeEnum::ColumnDef(column)) => Some(column.colname.clone()),
            _ => None,
        })
        .collect()
}
