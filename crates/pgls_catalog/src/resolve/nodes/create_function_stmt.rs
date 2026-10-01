//! Checks the final statement of a SQL function against its declared result, like Postgres does
//! when it creates the function (`42P13`).

use pgls_query::{
    NodeEnum,
    protobuf::{CreateFunctionStmt, FunctionParameter, FunctionParameterMode as Mode, TypeName},
};

use super::{Resolver, resolve_node_enum, string::string_values};
use crate::lookup::Lookup;
use crate::resolve::{FindingKind, FunctionContext, FunctionParam, ReturnMismatch};
use crate::typing::{
    CoercionContext, Decision, Type, TypeId, TypeKind, can_coerce, format_type_with_search_path,
    normalize_type_name,
};

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

enum Shape {
    /// Exactly one column of this type.
    Scalar(Option<TypeId>),
    /// One column per attribute, or a single column holding the whole row of type `row`.
    Row {
        row: Option<TypeId>,
        attributes: Vec<Option<TypeId>>,
    },
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
    let Some(found) = output_columns(r, n, &params, last) else {
        return;
    };
    if let Some(mismatch) = compare(r, &shape, &found) {
        r.report(
            FindingKind::FunctionReturnMismatch {
                declared: label,
                mismatch,
            },
            location,
        );
    }
}

/// How the final statement's columns differ from the declared result. Port of
/// `functions.c: check_sql_fn_retval` and `coerce_fn_result_column`: each column must be
/// assignable to its declared type.
fn compare(r: &Resolver, shape: &Shape, found: &[Option<Type>]) -> Option<ReturnMismatch> {
    // `None` if it is unknown whether the column can be assigned.
    let assignable = |found: &Option<Type>, expected: &Option<TypeId>| -> Option<bool> {
        let (found, expected) = (found.as_ref()?, expected.as_ref()?);
        match can_coerce(r.catalog, found, expected, CoercionContext::Assignment) {
            Decision::Known(assignable) => Some(assignable),
            Decision::Unknown => None,
        }
    };
    let column_type = |position: Option<usize>, expected: &Option<TypeId>, found: &Option<Type>| {
        Some(ReturnMismatch::ColumnType {
            position,
            expected: Type::Named(expected.clone()?),
            found: found.clone()?,
        })
    };
    match shape {
        Shape::Scalar(_) if found.len() != 1 => Some(ReturnMismatch::ColumnCount {
            expected: 1,
            found: found.len(),
        }),
        Shape::Scalar(expected) => match assignable(&found[0], expected)? {
            true => None,
            false => column_type(None, expected, &found[0]),
        },
        Shape::Row { row, attributes } => {
            // A single column may hold the whole row.
            if found.len() == 1 {
                match assignable(&found[0], row)? {
                    true => return None,
                    false if attributes.len() != 1 => {
                        return Some(ReturnMismatch::ColumnCount {
                            expected: attributes.len(),
                            found: 1,
                        });
                    }
                    false => {}
                }
            }
            if found.len() != attributes.len() {
                return Some(ReturnMismatch::ColumnCount {
                    expected: attributes.len(),
                    found: found.len(),
                });
            }
            for (position, (expected, found)) in attributes.iter().zip(found).enumerate() {
                match assignable(found, expected) {
                    Some(true) => {}
                    Some(false) => return column_type(Some(position + 1), expected, found),
                    None => return None,
                }
            }
            None
        }
    }
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
            shape: Some(Shape::Row {
                row: type_id(r, "record"),
                attributes: outputs
                    .iter()
                    .map(|output| normalize_type_name(r.catalog, output, r.search_path).0)
                    .collect(),
            }),
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
    let (id, _) = normalize_type_name(r.catalog, type_name, r.search_path);
    if let Some(id) = &id {
        label = format_type_with_search_path(r.catalog, &Type::Named(id.clone()), r.search_path)
            .unwrap_or(label);
    }
    let shape = if is_array {
        Some(Shape::Scalar(id))
    } else {
        match type_info.kind {
            // Postgres treats domains as scalars, even over composite types.
            Some(TypeKind::Composite) => {
                type_info.attributes.as_ref().map(|attributes| Shape::Row {
                    row: id,
                    attributes: attributes
                        .iter()
                        .map(|attribute| match &attribute.ty {
                            Some(Type::Named(ty)) => Some(ty.clone()),
                            _ => None,
                        })
                        .collect(),
                })
            }
            Some(
                TypeKind::Base
                | TypeKind::Domain
                | TypeKind::Enum
                | TypeKind::Range
                | TypeKind::Multirange,
            ) => Some(Shape::Scalar(id)),
            Some(TypeKind::Pseudo) => None,
            // Without metadata, attributes mean a composite type.
            None if type_info.attributes.is_some() => {
                type_info.attributes.as_ref().map(|attributes| Shape::Row {
                    row: id,
                    attributes: vec![None; attributes.len()],
                })
            }
            None if type_info.schema == "pg_catalog" => Some(Shape::Scalar(id)),
            None => None,
        }
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

fn type_id(r: &Resolver, name: &str) -> Option<TypeId> {
    match r.catalog.type_(Some("pg_catalog"), name, r.search_path) {
        Lookup::Found(info) => info.id,
        _ => None,
    }
}

/// The types of the columns the final statement returns, if the columns are known with
/// certainty. Unknown literals are `text`: Postgres resolves them before checking the result.
fn output_columns(
    r: &Resolver,
    n: &CreateFunctionStmt,
    params: &[&FunctionParameter],
    last: &NodeEnum,
) -> Option<Vec<Option<Type>>> {
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
    if !body.findings.is_empty() {
        return None;
    }
    let types: Vec<Option<Type>> = match body.output.take() {
        Some(output) if output.len() == columns.len() => {
            output.into_iter().map(|column| column.ty).collect()
        }
        _ => vec![None; columns.len()],
    };
    let text = type_id(r, "text").map(Type::Named);
    Some(
        types
            .into_iter()
            .map(|ty| match ty {
                Some(Type::UnknownLiteral) => text.clone(),
                ty => ty,
            })
            .collect(),
    )
}
