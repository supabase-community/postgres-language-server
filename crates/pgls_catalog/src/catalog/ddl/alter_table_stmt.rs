use pgls_query::{
    NodeEnum,
    protobuf::{AlterTableCmd, AlterTableStmt, AlterTableType, ObjectType},
};

use crate::catalog::{
    Catalog, Entry, key,
    names::{range_var_name, type_label},
    overlay::column_info,
};
use crate::view::{CatalogView, ColumnInfo, Origin};

/// `ALTER TABLE`, and `ALTER TYPE ... ADD/DROP/ALTER ATTRIBUTE` on composite types.
pub(super) fn apply_alter_table_stmt(c: &mut Catalog, n: &AlterTableStmt, search_path: &[String]) {
    let Some(range_var) = &n.relation else {
        return;
    };
    let commands: Vec<&AlterTableCmd> = n
        .cmds
        .iter()
        .filter_map(|command| match &command.node {
            Some(NodeEnum::AlterTableCmd(command)) => Some(command.as_ref()),
            _ => None,
        })
        .collect();

    if n.objtype() == ObjectType::ObjectType {
        let name = range_var_name(range_var);
        if let Some(mut type_info) = c.type_(name.schema(), &name.name, search_path).found() {
            if let Some(attributes) = type_info.attributes.as_mut() {
                for command in &commands {
                    apply_column_change(attributes, command);
                }
            }
            type_info.origin = Origin::File;
            let key = key(&type_info.schema, &type_info.name);
            c.types.insert(key, Entry::Defined(type_info));
        }
        return;
    }

    let Some(relation) = c.relation_of(range_var, search_path).found() else {
        return;
    };
    let changes_columns = commands.iter().any(|command| {
        matches!(
            command.subtype(),
            AlterTableType::AtAddColumn
                | AlterTableType::AtDropColumn
                | AlterTableType::AtAlterColumnType
        )
    });
    if !changes_columns {
        return;
    }

    c.change_columns(&relation, |columns| {
        if let Some(columns) = columns.as_mut() {
            for command in &commands {
                apply_column_change(columns, command);
            }
        }
    });
}

/// Applies `ADD COLUMN`, `DROP COLUMN` and `ALTER COLUMN TYPE` (and their `ATTRIBUTE`
/// counterparts).
fn apply_column_change(columns: &mut Vec<ColumnInfo>, command: &AlterTableCmd) {
    match command.subtype() {
        AlterTableType::AtAddColumn => {
            if let Some(NodeEnum::ColumnDef(column)) =
                command.def.as_deref().and_then(|def| def.node.as_ref())
            {
                if !columns.iter().any(|c| c.name == column.colname) {
                    columns.push(column_info(column));
                }
            }
        }
        AlterTableType::AtDropColumn => columns.retain(|column| column.name != command.name),
        AlterTableType::AtAlterColumnType => {
            if let (Some(NodeEnum::ColumnDef(definition)), Some(column)) = (
                command.def.as_deref().and_then(|def| def.node.as_ref()),
                columns
                    .iter_mut()
                    .find(|column| column.name == command.name),
            ) {
                column.type_name = definition.type_name.as_ref().and_then(type_label);
            }
        }
        _ => {}
    }
}
