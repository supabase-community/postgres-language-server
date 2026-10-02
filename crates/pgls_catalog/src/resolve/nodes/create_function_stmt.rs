//! Checks the final statement of a SQL function against its declared result, like Postgres does
//! when it creates the function (`42P13`).

use pgls_query::{
    NodeEnum,
    protobuf::{CreateFunctionStmt, FunctionParameter, FunctionParameterMode as Mode, TypeName},
};

use super::{Resolver, resolve_node_enum, string::string_values};
use crate::lookup::Lookup;
use crate::resolve::{FindingKind, FunctionContext, FunctionParam, ReturnMismatch};

/// Pseudo-types. Functions returning them have no fixed shape, and polymorphic arguments make
/// Postgres skip the check of the body.
const PSEUDO_TYPES: &[&str] = &[
    "any",
    "anyarray",
    "anycompatible",
    "anycompatiblearray",
    "anycompatiblemultirange",
    "anycompatiblenonarray",
    "anycompatiblerange",
    "anyelement",
    "anyenum",
    "anymultirange",
    "anynonarray",
    "anyrange",
    "cstring",
    "event_trigger",
    "fdw_handler",
    "index_am_handler",
    "internal",
    "language_handler",
    "opaque",
    "record",
    "table_am_handler",
    "trigger",
    "tsm_handler",
    "unknown",
];

/// The result a function is declared to return.
enum Declared {
    Void,
    Value {
        /// The type as shown in messages.
        label: String,
        /// The number of columns the final statement must return, if known.
        shape: Option<Shape>,
        /// Where to report a mismatch.
        location: i32,
    },
}

#[derive(Clone, Copy)]
enum Shape {
    /// Exactly one column.
    Scalar,
    /// One column per attribute, or a single column holding the whole row.
    Row(usize),
}

pub(super) fn resolve_create_function_stmt(r: &mut Resolver, n: &CreateFunctionStmt) {
    // Statements of the file never come unchanged from the database.
    r.depends_on_file();

    if n.is_procedure || !is_sql_function(n) {
        return;
    }
    let params: Vec<&FunctionParameter> = n
        .parameters
        .iter()
        .filter_map(|param| match &param.node {
            Some(NodeEnum::FunctionParameter(param)) => Some(param.as_ref()),
            _ => None,
        })
        .collect();
    // Postgres can't check the body of functions with polymorphic arguments.
    if params
        .iter()
        .any(|param| param.arg_type.as_ref().is_none_or(is_pseudo_type))
    {
        return;
    }
    let Some(Declared::Value {
        label,
        shape,
        location,
    }) = declared_result(r, n, &params)
    else {
        return;
    };
    let Some(statements) = body_statements(n) else {
        return;
    };
    let Some(last) = statements.last() else {
        return;
    };

    let returns_rows = match last {
        // `SELECT ... INTO` is rejected for other reasons.
        NodeEnum::SelectStmt(select) if select.into_clause.is_some() => return,
        NodeEnum::SelectStmt(_) => true,
        NodeEnum::InsertStmt(insert) => !insert.returning_list.is_empty(),
        NodeEnum::UpdateStmt(update) => !update.returning_list.is_empty(),
        NodeEnum::DeleteStmt(delete) => !delete.returning_list.is_empty(),
        NodeEnum::MergeStmt(merge) => !merge.returning_list.is_empty(),
        _ => false,
    };
    if !returns_rows {
        r.report(
            FindingKind::FunctionReturnMismatch {
                declared: label,
                mismatch: ReturnMismatch::NoRows,
            },
            location,
        );
        return;
    }

    let Some(shape) = shape else {
        return;
    };
    let Some(found) = output_column_count(r, n, &params, last) else {
        return;
    };
    let expected = match shape {
        Shape::Scalar if found != 1 => 1,
        Shape::Row(columns) if found != 1 && found != columns => columns,
        _ => return,
    };
    r.report(
        FindingKind::FunctionReturnMismatch {
            declared: label,
            mismatch: ReturnMismatch::ColumnCount { expected, found },
        },
        location,
    );
}

fn is_sql_function(n: &CreateFunctionStmt) -> bool {
    // `BEGIN ATOMIC` and `RETURN` bodies only exist for SQL functions.
    n.sql_body.is_some()
        || option_string(n, "language").is_some_and(|language| language.eq_ignore_ascii_case("sql"))
}

/// The string value of a function option, like `language` or `as`.
fn option_string<'a>(n: &'a CreateFunctionStmt, name: &str) -> Option<&'a str> {
    n.options.iter().find_map(|option| {
        let Some(NodeEnum::DefElem(option)) = &option.node else {
            return None;
        };
        if option.defname != name {
            return None;
        }
        match option.arg.as_deref()?.node.as_ref()? {
            NodeEnum::String(value) => Some(value.sval.as_str()),
            NodeEnum::List(list) => match list.items.as_slice() {
                [item] => match item.node.as_ref()? {
                    NodeEnum::String(value) => Some(value.sval.as_str()),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        }
    })
}

/// The statements of the body. `None` if it can't be read, and for `RETURN expression`
/// bodies, which always return exactly one column.
fn body_statements(n: &CreateFunctionStmt) -> Option<Vec<NodeEnum>> {
    if let Some(body) = &n.sql_body {
        // `BEGIN ATOMIC ... END` is a list holding the list of statements.
        let NodeEnum::List(outer) = body.node.as_ref()? else {
            return None;
        };
        let NodeEnum::List(statements) = outer.items.first()?.node.as_ref()? else {
            return None;
        };
        return statements
            .items
            .iter()
            .map(|statement| statement.node.clone())
            .collect();
    }
    let body = option_string(n, "as")?;
    let parsed = pgls_query::parse(body).ok()?;
    Some(parsed.stmts().into_iter().cloned().collect())
}

fn declared_result(
    r: &Resolver,
    n: &CreateFunctionStmt,
    params: &[&FunctionParameter],
) -> Option<Declared> {
    let outputs: Vec<&TypeName> = params
        .iter()
        .filter(|param| {
            matches!(
                param.mode(),
                Mode::FuncParamOut | Mode::FuncParamInout | Mode::FuncParamTable
            )
        })
        .filter_map(|param| param.arg_type.as_ref())
        .collect();
    match outputs.as_slice() {
        // Without output parameters, the return type decides.
        [] => declared_type(r, n.return_type.as_ref()?),
        // A single output parameter is the result.
        [output] => declared_type(r, output),
        [first, ..] => Some(Declared::Value {
            label: "record".into(),
            shape: Some(Shape::Row(outputs.len())),
            location: first.location,
        }),
    }
}

fn declared_type(r: &Resolver, type_name: &TypeName) -> Option<Declared> {
    let names = string_values(&type_name.names)?;
    let (schema, name) = match names.as_slice() {
        [name] => (None, name.as_str()),
        [schema, name] => (Some(schema.as_str()), name.as_str()),
        _ => return None,
    };
    let is_array = !type_name.array_bounds.is_empty();
    let builtin = schema.is_none_or(|schema| schema == "pg_catalog");
    let mut label = match schema {
        Some(schema) if schema != "pg_catalog" => format!("{schema}.{name}"),
        _ => name.to_owned(),
    };
    // The type of `table.column%TYPE` is not resolved.
    if type_name.pct_type {
        return None;
    }
    if is_array {
        label.push_str("[]");
    }
    if builtin && !is_array && name == "void" {
        return Some(Declared::Void);
    }
    if builtin && !is_array && PSEUDO_TYPES.contains(&name) {
        return Some(Declared::Value {
            label,
            shape: None,
            location: type_name.location,
        });
    }

    // A missing type is reported by the `unknownType` rule.
    let Lookup::Found(type_info) = r.catalog.type_(schema, name, r.search_path) else {
        return None;
    };
    let shape = if is_array {
        Some(Shape::Scalar)
    } else if let Some(attributes) = &type_info.attributes {
        Some(Shape::Row(attributes.len()))
    } else if type_info.schema == "pg_catalog" {
        Some(Shape::Scalar)
    } else {
        // Enums, domains, ranges, and composite types with unknown attributes.
        None
    };
    Some(Declared::Value {
        label,
        shape,
        location: type_name.location,
    })
}

fn is_pseudo_type(type_name: &TypeName) -> bool {
    let Some(names) = string_values(&type_name.names) else {
        return true;
    };
    match names.as_slice() {
        [name] => PSEUDO_TYPES.contains(&name.as_str()),
        [schema, name] => schema == "pg_catalog" && PSEUDO_TYPES.contains(&name.as_str()),
        _ => false,
    }
}

/// The number of columns the final statement returns, if it is known with certainty.
fn output_column_count(
    r: &Resolver,
    n: &CreateFunctionStmt,
    params: &[&FunctionParameter],
    last: &NodeEnum,
) -> Option<usize> {
    let function = FunctionContext {
        function_name: string_values(&n.funcname)?.pop()?,
        params: params
            .iter()
            .filter(|param| !matches!(param.mode(), Mode::FuncParamOut | Mode::FuncParamTable))
            .map(|param| {
                let type_name = param.arg_type.as_ref()?;
                let mut names = string_values(&type_name.names)?;
                Some(FunctionParam {
                    name: (!param.name.is_empty()).then(|| param.name.clone()),
                    type_name: names.pop()?,
                    type_schema: names.pop(),
                    is_array: !type_name.array_bounds.is_empty(),
                })
            })
            .collect::<Option<_>>()?,
    };
    // The body is resolved on its own, and its findings are reported for its statements.
    let mut body = Resolver::new(r.catalog, r.search_path, Some(&function), None);
    let columns = resolve_node_enum(&mut body, last)?;
    body.findings.is_empty().then_some(columns.len())
}
