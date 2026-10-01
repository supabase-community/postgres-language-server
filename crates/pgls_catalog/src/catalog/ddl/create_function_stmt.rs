use pgls_query::{
    NodeEnum,
    protobuf::{CreateFunctionStmt, FunctionParameterMode as Mode},
};

use crate::catalog::{
    Catalog,
    names::{qualified_name, type_label, type_name},
    overlay::function_info,
};
use crate::lookup::{CatalogView, ColumnInfo, FunctionKind};

/// `CREATE FUNCTION` and `CREATE PROCEDURE` add an overload.
pub(super) fn apply_create_function_stmt(
    c: &mut Catalog,
    n: &CreateFunctionStmt,
    search_path: &[String],
) {
    let Some(name) = qualified_name(&n.funcname) else {
        return;
    };
    let (schema, name) = c.creation_key(&name, search_path);

    let mut inputs = 0;
    let mut defaults = 0;
    let mut variadic = false;
    let mut outputs = Vec::new();
    for parameter in &n.parameters {
        let Some(NodeEnum::FunctionParameter(parameter)) = &parameter.node else {
            continue;
        };
        let mode = parameter.mode();
        if matches!(
            mode,
            Mode::FuncParamOut | Mode::FuncParamInout | Mode::FuncParamTable
        ) {
            outputs.push(ColumnInfo {
                name: parameter.name.clone(),
                type_name: parameter.arg_type.as_ref().and_then(type_label),

                ty: None,
            });
        }
        if matches!(mode, Mode::FuncParamOut | Mode::FuncParamTable) {
            continue;
        }
        inputs += 1;
        variadic |= mode == Mode::FuncParamVariadic;
        if parameter.defexpr.is_some() {
            defaults += 1;
        }
    }

    let return_type = n.return_type.as_ref();
    let returns_set = return_type.is_some_and(|return_type| return_type.setof);
    let return_columns = if outputs.is_empty() {
        return_type
            .and_then(type_name)
            .and_then(|name| c.type_(name.schema(), &name.name, search_path).found())
            .and_then(|type_info| type_info.attributes)
    } else {
        Some(outputs)
    };

    let kind = if n.is_procedure {
        FunctionKind::Procedure
    } else {
        FunctionKind::Function
    };
    let mut function = function_info(
        &schema,
        &name,
        kind,
        inputs - defaults,
        (!variadic).then_some(inputs),
    );
    function.returns_set = returns_set;
    function.return_columns = return_columns;
    c.add_function(function);
}
