use pgls_query::{
    NodeEnum,
    protobuf::{AlterObjectSchemaStmt, ObjectType},
};

use crate::catalog::{Catalog, names::qualified_name};

/// `ALTER ... SET SCHEMA`. Models [`ExecAlterObjectSchemaStmt`].
///
/// [`ExecAlterObjectSchemaStmt`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/alter.c#L534
pub(super) fn apply_alter_object_schema_stmt(
    c: &mut Catalog,
    n: &AlterObjectSchemaStmt,
    search_path: &[String],
) {
    let object = n.object.as_deref().and_then(|object| object.node.as_ref());
    let new_schema = n.newschema.as_str();
    match n.object_type() {
        ObjectType::ObjectTable
        | ObjectType::ObjectView
        | ObjectType::ObjectMatview
        | ObjectType::ObjectForeignTable
        | ObjectType::ObjectSequence => {
            if let Some(range_var) = &n.relation {
                c.move_relation(range_var, Some(new_schema), None, search_path);
            }
        }
        ObjectType::ObjectType | ObjectType::ObjectDomain => {
            if let Some(NodeEnum::List(list)) = object {
                if let Some(name) = qualified_name(&list.items) {
                    c.move_type(&name, Some(new_schema), None, search_path);
                }
            }
        }
        ObjectType::ObjectFunction
        | ObjectType::ObjectProcedure
        | ObjectType::ObjectRoutine
        | ObjectType::ObjectAggregate => {
            if let Some(NodeEnum::ObjectWithArgs(function)) = object {
                c.move_function(function, Some(new_schema), None, search_path);
            }
        }
        // Moves all objects of the extension.
        ObjectType::ObjectExtension => c.tainted = true,
        _ => {}
    }
}
