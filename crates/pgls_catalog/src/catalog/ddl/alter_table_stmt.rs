use pgls_query::{
    NodeEnum,
    protobuf::{AlterTableCmd, AlterTableStmt, AlterTableType, ObjectType},
};

use crate::Generated;
use crate::catalog::{
    Catalog, Entry, key,
    names::{range_var_name, type_label},
    overlay::column_info,
};
use crate::lookup::{CatalogView, ColumnInfo, Origin, RelationInfo};

/// `ALTER TABLE`, and `ALTER TYPE ... ADD/DROP/ALTER ATTRIBUTE` on composite types.
/// Models the column changes of [`ATExecAddColumn`], [`ATExecDropColumn`],
/// [`ATExecAlterColumnType`] and [`ATExecDropExpression`], and which tables inherit from which
/// (`ATTACH PARTITION`, `INHERIT`, ...).
///
/// [`ATExecAddColumn`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/tablecmds.c#L7226
/// [`ATExecDropColumn`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/tablecmds.c#L9291
/// [`ATExecAlterColumnType`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/tablecmds.c#L14747
/// [`ATExecDropExpression`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/tablecmds.c#L8809
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
        if let Some(mut type_info) = c
            .type_(name.schema(), &name.name, search_path)
            .found()
            .filter(|type_info| type_info.schema != "pg_catalog")
        {
            if let Some(attributes) = type_info.attributes.as_mut() {
                for command in &commands {
                    apply_column_change(c, attributes, command, search_path);
                }
            }
            type_info.origin = Origin::File;
            let key = key(&type_info.schema, &type_info.name);
            c.types.insert(key, Entry::Defined(type_info));
        }
        return;
    }

    // Postgres rejects changes to system catalogs.
    let Some(relation) = c
        .relation_of(range_var, search_path)
        .found()
        .filter(|relation| relation.schema != "pg_catalog")
    else {
        return;
    };
    apply_column_changes(c, &relation, &commands, search_path);
    if n.objtype() == ObjectType::ObjectTable || n.objtype() == ObjectType::ObjectForeignTable {
        apply_inheritance_changes(c, &relation, &commands, search_path);
    }
}

/// The column changes of `ALTER TABLE`, which recurse to the inheritance children and partitions.
fn apply_column_changes(
    c: &mut Catalog,
    relation: &RelationInfo,
    commands: &[&AlterTableCmd],
    search_path: &[String],
) {
    let changes_columns = commands.iter().any(|command| {
        matches!(
            command.subtype(),
            AlterTableType::AtAddColumn
                | AlterTableType::AtDropColumn
                | AlterTableType::AtAlterColumnType
                | AlterTableType::AtDropExpression
        )
    });
    if !changes_columns {
        return;
    }

    // Postgres rejects dropping or retyping an inherited column, and we don't know which
    // columns of a child are inherited.
    let changes_inherited = commands.iter().any(|command| {
        matches!(
            command.subtype(),
            AlterTableType::AtDropColumn | AlterTableType::AtAlterColumnType
        )
    });
    if changes_inherited && c.is_inheritance_child(relation) {
        c.change_columns(relation, |columns| *columns = None);
        return;
    }
    let catalog = c.clone();
    c.change_columns(relation, |columns| {
        if let Some(columns) = columns.as_mut() {
            for command in commands {
                apply_column_change(&catalog, columns, command, search_path);
            }
        }
    });

    // A child keeps a dropped column that is also its own or inherited from another parent, and
    // so do its own children ([`ATExecDropColumn`]). We don't know which columns those are.
    //
    // [`ATExecDropColumn`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/tablecmds.c#L9419
    let drops_columns = commands
        .iter()
        .any(|command| command.subtype() == AlterTableType::AtDropColumn);
    if drops_columns {
        let root = key(&relation.schema, &relation.name);
        let keeping: Vec<_> = c
            .with_descendants(root.clone())
            .into_iter()
            .filter(|child| *child != root && c.children_with_own_columns.contains(child))
            .flat_map(|child| c.with_descendants(child))
            .collect();
        for child in keeping {
            if let Some(mut child) = c.relation_in(&child.0, &child.1).found() {
                child.columns = None;
                child.origin = Origin::File;
                c.define_relation(child);
            }
        }
    }
}

/// `ATTACH PARTITION` and `INHERIT` make a table a child of another, so that it follows the
/// column changes of its parent from then on. `DETACH PARTITION` and `NO INHERIT` undo that; the
/// table keeps its columns ([`ATExecAttachPartition`], [`ATExecAddInherit`],
/// [`ATExecDetachPartition`], [`ATExecDropInherit`]). Postgres runs them after the column
/// changes of the same statement, and decides to which children those recurse before.
///
/// [`ATExecAttachPartition`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/tablecmds.c#L20297
/// [`ATExecAddInherit`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/tablecmds.c#L17303
/// [`ATExecDetachPartition`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/tablecmds.c#L20959
/// [`ATExecDropInherit`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/tablecmds.c#L17867
fn apply_inheritance_changes(
    c: &mut Catalog,
    relation: &RelationInfo,
    commands: &[&AlterTableCmd],
    search_path: &[String],
) {
    let relation_key = key(&relation.schema, &relation.name);
    for command in commands {
        let def = command.def.as_deref().and_then(|def| def.node.as_ref());
        let (other, relation_is_parent, attach) = match (command.subtype(), def) {
            (AlterTableType::AtAttachPartition, Some(NodeEnum::PartitionCmd(partition))) => {
                (partition.name.as_ref(), true, true)
            }
            (
                AlterTableType::AtDetachPartition | AlterTableType::AtDetachPartitionFinalize,
                Some(NodeEnum::PartitionCmd(partition)),
            ) => (partition.name.as_ref(), true, false),
            (AlterTableType::AtAddInherit, Some(NodeEnum::RangeVar(parent))) => {
                (Some(parent), false, true)
            }
            (AlterTableType::AtDropInherit, Some(NodeEnum::RangeVar(parent))) => {
                (Some(parent), false, false)
            }
            _ => continue,
        };
        let Some(other) = other.and_then(|other| c.relation_of(other, search_path).found()) else {
            continue;
        };
        let other_key = key(&other.schema, &other.name);
        let (parent, child) = if relation_is_parent {
            (relation_key.clone(), other_key)
        } else {
            (other_key, relation_key.clone())
        };
        // Postgres rejects circular inheritance.
        if attach && c.with_descendants(child.clone()).contains(&parent) {
            continue;
        }
        // The columns of a table that becomes an inheritance child stay its own. A partition's
        // columns become inherited only.
        if attach && !relation_is_parent {
            c.children_with_own_columns.insert(child.clone());
        }
        let children = c.children.entry(parent).or_default();
        if attach {
            if !children.contains(&child) {
                children.push(child);
            }
        } else {
            children.retain(|existing| *existing != child);
        }
    }
}

/// Applies `ADD COLUMN`, `DROP COLUMN` and `ALTER COLUMN TYPE` (and their `ATTRIBUTE`
/// counterparts), and `ALTER COLUMN DROP EXPRESSION`, which turns a generated column into an
/// ordinary one.
fn apply_column_change(
    c: &Catalog,
    columns: &mut Vec<ColumnInfo>,
    command: &AlterTableCmd,
    search_path: &[String],
) {
    match command.subtype() {
        AlterTableType::AtAddColumn => {
            if let Some(NodeEnum::ColumnDef(column)) =
                command.def.as_deref().and_then(|def| def.node.as_ref())
                && !columns.iter().any(|c| c.name == column.colname)
            {
                columns.push(column_info(c, column, search_path));
            }
        }
        AlterTableType::AtDropColumn => columns.retain(|column| column.name != command.name),
        AlterTableType::AtDropExpression => {
            if let Some(column) = columns
                .iter_mut()
                .find(|column| column.name == command.name)
            {
                column.generated = Generated::No;
            }
        }
        AlterTableType::AtAlterColumnType => {
            if let (Some(NodeEnum::ColumnDef(definition)), Some(column)) = (
                command.def.as_deref().and_then(|def| def.node.as_ref()),
                columns
                    .iter_mut()
                    .find(|column| column.name == command.name),
            ) {
                column.type_name = definition.type_name.as_ref().and_then(type_label);
                column.ty = definition
                    .type_name
                    .as_ref()
                    .and_then(|name| crate::normalize_type_name(c, name, search_path).0)
                    .map(crate::typing::Type::Named);
            }
        }
        _ => {}
    }
}
