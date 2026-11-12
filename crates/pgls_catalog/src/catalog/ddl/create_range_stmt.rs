use pgls_query::protobuf::CreateRangeStmt;
use pgls_query_ext::options::find_def_elem;

use crate::catalog::{Catalog, names::qualified_name, overlay::function_info};
use crate::lookup::FunctionKind;

/// A range type also creates its multirange type and their constructor functions. Models
/// [`DefineRange`].
///
/// [`DefineRange`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/typecmds.c#L1388
pub(super) fn apply_create_range_stmt(
    c: &mut Catalog,
    n: &CreateRangeStmt,
    search_path: &[String],
) {
    let Some(name) = qualified_name(&n.type_name) else {
        return;
    };
    if find_def_elem(&n.params, "multirange_type_name").is_some() {
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
        c.define_type_with_metadata(
            schema.clone(),
            type_name.clone(),
            None,
            if type_name == &range_name {
                crate::TypeKind::Range
            } else {
                crate::TypeKind::Multirange
            },
            if type_name == &range_name { 'R' } else { 'M' },
            None,
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
