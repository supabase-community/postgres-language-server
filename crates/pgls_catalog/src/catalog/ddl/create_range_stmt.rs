use pgls_query::{NodeEnum, protobuf::CreateRangeStmt};

use crate::catalog::{
    Catalog,
    names::{QualifiedName, qualified_name},
    overlay::function_info,
};
use crate::lookup::FunctionKind;

/// A range type also creates its multirange type and their constructor functions.
pub(super) fn apply_create_range_stmt(
    c: &mut Catalog,
    n: &CreateRangeStmt,
    search_path: &[String],
) {
    let Some(name) = qualified_name(&n.type_name) else {
        return;
    };
    let custom_multirange_name = n.params.iter().any(|param| {
        matches!(&param.node, Some(NodeEnum::DefElem(param)) if param.defname == "multirange_type_name")
    });
    if custom_multirange_name {
        c.tainted = true;
        return;
    }

    let (schema, range_name) = c.creation_key(&name, search_path);
    let multirange_name = if range_name.contains("range") {
        range_name.replacen("range", "multirange", 1)
    } else {
        format!("{range_name}_multirange")
    };

    for type_name in [&range_name, &multirange_name] {
        c.define_type(
            &QualifiedName {
                schema: Some(schema.clone()),
                name: type_name.clone(),
            },
            None,
            search_path,
        );
    }
    // `range(lower, upper [, bounds])` and `multirange(VARIADIC ranges)`.
    c.add_function(function_info(
        &schema,
        &range_name,
        FunctionKind::Function,
        2,
        Some(3),
    ));
    c.add_function(function_info(
        &schema,
        &multirange_name,
        FunctionKind::Function,
        0,
        None,
    ));
}
