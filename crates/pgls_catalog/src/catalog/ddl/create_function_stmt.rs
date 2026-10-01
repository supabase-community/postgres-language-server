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
            Mode::FuncParamInout => crate::typing::FunctionArgumentMode::InOut,
            Mode::FuncParamVariadic => crate::typing::FunctionArgumentMode::Variadic,
            _ => crate::typing::FunctionArgumentMode::In,
        };
        let is_output = matches!(mode, Mode::FuncParamOut | Mode::FuncParamTable);
        // Postgres rejects defaults for output parameters, and input parameters without a
        // default after one with a default (`interpret_function_parameter_list`).
        if parameter.defexpr.is_some() && is_output
            || parameter.defexpr.is_none() && !is_output && defaults > 0
        {
            return;
        }
        if is_output {
            continue;
        }
        arguments.push(crate::typing::FunctionArgument {
            name: (!parameter.name.is_empty()).then(|| parameter.name.clone()),
            ty: argument_type,
            mode: argument_mode,
        });
        inputs += 1;
        variadic |= mode == Mode::FuncParamVariadic;
        if parameter.defexpr.is_some() {
            defaults += 1;
        }
    }

    let output_types: Vec<Option<crate::typing::TypeId>> = outputs
        .iter()
        .map(|output| match &output.ty {
            Some(crate::typing::Type::Named(ty)) => Some(ty.clone()),
            _ => None,
        })
        .collect();
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
    // Postgres rejects polymorphic results that the inputs can't determine (`ProcedureCreate`).
    let input_types: Option<Vec<Option<&str>>> = arguments
        .iter()
        .map(|argument| polymorphic_name(c, argument.ty.as_ref()?))
        .collect();
    if let Some(input_types) = input_types {
        let results = std::iter::once(return_type.as_ref())
            .chain(output_types.iter().map(Option::as_ref))
            .flatten();
        for result in results {
            if let Some(Some(result)) = polymorphic_name(c, result) {
                if !valid_polymorphic_result(result, &input_types) {
                    return;
                }
            }
        }
    }
    let variadic_type = n
        .parameters
        .iter()
        .find_map(|parameter| match parameter.node.as_ref() {
            Some(NodeEnum::FunctionParameter(parameter))
                if parameter.mode() == Mode::FuncParamVariadic =>
            {
                Some(parameter.arg_type.as_ref())
            }
            _ => None,
        });
    let variadic_element = match variadic_type {
        None => Some(None),
        Some(ty) => ty
            .and_then(|ty| crate::normalize_type_name(c, ty, search_path).0)
            .and_then(|id| variadic_element(c, &id))
            .map(Some),
    };
    // A variadic function whose element type is unknown can't be matched.
    function.signature =
        variadic_element.map(|variadic_element| crate::typing::FunctionSignature {
            arguments,
            input_defaults: defaults,
            variadic_element,
            return_type,
            returns_set,
        });
    c.add_function(function);
}

/// The element type of a variadic parameter. Port of the `provariadic` logic in
/// `functioncmds.c: interpret_function_parameter_list`.
fn variadic_element(c: &Catalog, id: &crate::typing::TypeId) -> Option<crate::typing::TypeId> {
    let info = c.type_by_id(id).found()?;
    if info.schema == "pg_catalog" {
        let element = match info.name.as_str() {
            "anyarray" => Some("anyelement"),
            "anycompatiblearray" => Some("anycompatible"),
            "any" => Some("any"),
            _ => None,
        };
        if let Some(element) = element {
            return c.type_(Some("pg_catalog"), element, &[]).found()?.id;
        }
    }
    info.element
}

/// The name of a type if it is polymorphic, `Some(None)` if it is not, and `None` if the type
/// is unknown.
fn polymorphic_name(c: &Catalog, id: &crate::typing::TypeId) -> Option<Option<&'static str>> {
    const POLYMORPHIC: &[&str] = &[
        "anyelement",
        "anyarray",
        "anynonarray",
        "anyenum",
        "anyrange",
        "anymultirange",
        "anycompatible",
        "anycompatiblearray",
        "anycompatiblenonarray",
        "anycompatiblerange",
        "anycompatiblemultirange",
    ];
    let info = c.type_by_id(id).found()?;
    if info.schema != "pg_catalog" {
        return Some(None);
    }
    Some(POLYMORPHIC.iter().copied().find(|name| *name == info.name))
}

/// Whether the polymorphic result type can be determined from the input types. Port of
/// `parse_coerce.c: check_valid_polymorphic_signature`.
fn valid_polymorphic_result(result: &str, inputs: &[Option<&str>]) -> bool {
    let family1 = |name: &str| {
        matches!(
            name,
            "anyelement" | "anyarray" | "anynonarray" | "anyenum" | "anyrange" | "anymultirange"
        )
    };
    let accepts: &dyn Fn(&str) -> bool = match result {
        "anyrange" | "anymultirange" => &|name| matches!(name, "anyrange" | "anymultirange"),
        "anycompatiblerange" | "anycompatiblemultirange" => {
            &|name| matches!(name, "anycompatiblerange" | "anycompatiblemultirange")
        }
        name if family1(name) => &family1,
        _ => &|name: &str| name.starts_with("anycompatible"),
    };
    inputs.iter().flatten().any(|input| accepts(input))
}
