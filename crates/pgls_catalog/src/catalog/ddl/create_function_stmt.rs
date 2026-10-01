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
    let mut arguments = Vec::new();
    for parameter in &n.parameters {
        let Some(NodeEnum::FunctionParameter(parameter)) = &parameter.node else {
            continue;
        };
        let mode = parameter.mode();
        if matches!(
            mode,
            Mode::FuncParamOut | Mode::FuncParamInout | Mode::FuncParamTable
        ) {
            let argument_type = parameter
                .arg_type
                .as_ref()
                .and_then(|ty| crate::normalize_type_name(c, ty, search_path).0);
            outputs.push(ColumnInfo {
                name: parameter.name.clone(),
                type_name: parameter.arg_type.as_ref().and_then(type_label),
                ty: argument_type.clone().map(crate::typing::Type::Named),
            });
        }
        let argument_type = parameter
            .arg_type
            .as_ref()
            .and_then(|ty| crate::normalize_type_name(c, ty, search_path).0);
        let argument_mode = match mode {
            Mode::FuncParamOut | Mode::FuncParamTable => crate::typing::FunctionArgumentMode::In,
            Mode::FuncParamInout => crate::typing::FunctionArgumentMode::InOut,
            Mode::FuncParamVariadic => crate::typing::FunctionArgumentMode::Variadic,
            _ => crate::typing::FunctionArgumentMode::In,
        };
        arguments.push(crate::typing::FunctionArgument {
            name: (!parameter.name.is_empty()).then(|| parameter.name.clone()),
            ty: argument_type,
            mode: argument_mode,
        });
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
    let has_outputs = !outputs.is_empty();
    let return_columns = if !has_outputs {
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
    let return_type = if !has_outputs {
        n.return_type
            .as_ref()
            .and_then(|ty| crate::normalize_type_name(c, ty, search_path).0)
    } else {
        None
    };
    function.signature = Some(crate::typing::FunctionSignature {
        arguments,
        input_defaults: defaults,
        variadic_element: n
            .parameters
            .iter()
            .filter_map(|parameter| match parameter.node.as_ref() {
                Some(NodeEnum::FunctionParameter(parameter))
                    if parameter.mode() == Mode::FuncParamVariadic =>
                {
                    parameter
                        .arg_type
                        .as_ref()
                        .and_then(|ty| crate::normalize_type_name(c, ty, search_path).0)
                }
                _ => None,
            })
            .next(),
        return_type,
        returns_set,
    });
    c.add_function(function);
}
