use pgls_query::protobuf::{DefineStmt, ObjectType};

use crate::catalog::{Catalog, names::qualified_name, overlay::function_info};
use crate::lookup::FunctionKind;

/// `CREATE AGGREGATE`, `CREATE TYPE name (...)` and shell types. Models [`DefineType`].
///
/// [`DefineType`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/typecmds.c#L153
pub(super) fn apply_define_stmt(c: &mut Catalog, n: &DefineStmt, search_path: &[String]) {
    let Some(name) = qualified_name(&n.defnames) else {
        return;
    };
    match n.kind() {
        ObjectType::ObjectType => c.define_type(&name, None, search_path),
        ObjectType::ObjectOperator => c.operators_incomplete = true,
        ObjectType::ObjectAggregate => {
            let (schema, name) = c.creation_key(&name, search_path);
            // The arity of aggregates (ordered-set, `*`, variadic) is not modelled.
            c.add_function(function_info(
                &schema,
                &name,
                FunctionKind::Aggregate,
                0,
                None,
            ));
        }
        _ => {}
    }
}
