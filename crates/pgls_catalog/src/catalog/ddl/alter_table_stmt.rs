use pgls_query::{
    NodeEnum,
    protobuf::{AlterTableCmd, AlterTableStmt, AlterTableType, ObjectType},
};

use crate::catalog::{
    Catalog, Entry, key,
    names::{range_var_name, type_label},
    overlay::{column_info, identity_kind},
};
use crate::lookup::{CatalogView, ColumnInfo, Origin, RelationKind};
use crate::{Generated, Identity};

/// `ALTER TABLE`, and `ALTER TYPE ... ADD/DROP/ALTER ATTRIBUTE` on composite types.
/// Models the column changes of [`ATExecAddColumn`], [`ATExecDropColumn`],
/// [`ATExecAlterColumnType`] and [`ATExecDropExpression`].
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
                    apply_column_change(c, attributes, command, search_path, true);
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
    let changes_columns = commands.iter().any(|command| {
        matches!(
            command.subtype(),
            AlterTableType::AtAddColumn
                | AlterTableType::AtDropColumn
                | AlterTableType::AtAlterColumnType
                | AlterTableType::AtDropExpression
                | AlterTableType::AtAddIdentity
                | AlterTableType::AtSetIdentity
                | AlterTableType::AtDropIdentity
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
    if changes_inherited && c.is_inheritance_child(&relation) {
        c.change_columns(&relation, |columns, _| *columns = None);
        return;
    }
    // Partitions follow the identity changes of their partitioned table on Postgres 17+.
    // Inheritance children never do ([`ATExecAddIdentity`], [`ATPrepAddColumn`]).
    //
    // [`ATExecAddIdentity`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/tablecmds.c#L8354
    // [`ATPrepAddColumn`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/tablecmds.c#L7352
    let partitions_follow_identity =
        relation.kind == RelationKind::PartitionedTable && c.partitions_inherit_identity();
    let catalog = c.clone();
    c.change_columns(&relation, |columns, is_descendant| {
        if let Some(columns) = columns.as_mut() {
            let identity_follows = !is_descendant || partitions_follow_identity;
            for command in &commands {
                apply_column_change(&catalog, columns, command, search_path, identity_follows);
            }
        }
    });
}

/// Applies `ADD COLUMN`, `DROP COLUMN` and `ALTER COLUMN TYPE` (and their `ATTRIBUTE`
/// counterparts), `ALTER COLUMN DROP EXPRESSION`, which turns a generated column into an
/// ordinary one, and the identity changes `ADD GENERATED ... AS IDENTITY`, `SET GENERATED` and
/// `DROP IDENTITY`. Without `identity_follows`, the identity of the changed columns is unknown.
fn apply_column_change(
    c: &Catalog,
    columns: &mut Vec<ColumnInfo>,
    command: &AlterTableCmd,
    search_path: &[String],
    identity_follows: bool,
) {
    let set_identity = |columns: &mut Vec<ColumnInfo>, identity: Identity| {
        if let Some(column) = columns
            .iter_mut()
            .find(|column| column.name == command.name)
        {
            column.identity = if identity_follows {
                identity
            } else {
                Identity::No
            };
        }
    };
    match command.subtype() {
        AlterTableType::AtAddColumn => {
            if let Some(NodeEnum::ColumnDef(column)) =
                command.def.as_deref().and_then(|def| def.node.as_ref())
                && !columns.iter().any(|c| c.name == column.colname)
            {
                let mut column = column_info(c, column, search_path);
                if !identity_follows {
                    column.identity = Identity::No;
                }
                columns.push(column);
            }
        }
        AlterTableType::AtDropColumn => columns.retain(|column| column.name != command.name),
        AlterTableType::AtAddIdentity => {
            if let Some(NodeEnum::Constraint(constraint)) =
                command.def.as_deref().and_then(|def| def.node.as_ref())
            {
                set_identity(columns, identity_kind(&constraint.generated_when));
            }
        }
        // `SET GENERATED { ALWAYS | BY DEFAULT }` among the sequence options.
        AlterTableType::AtSetIdentity => {
            let generated = match command.def.as_deref().and_then(|def| def.node.as_ref()) {
                Some(NodeEnum::List(options)) => {
                    options.items.iter().find_map(|option| match &option.node {
                        Some(NodeEnum::DefElem(option)) if option.defname == "generated" => {
                            match option.arg.as_deref().and_then(|arg| arg.node.as_ref()) {
                                Some(NodeEnum::Integer(value)) => u8::try_from(value.ival).ok(),
                                _ => None,
                            }
                        }
                        _ => None,
                    })
                }
                _ => None,
            };
            if let Some(generated) = generated {
                set_identity(columns, identity_kind(&char::from(generated).to_string()));
            }
        }
        AlterTableType::AtDropIdentity => set_identity(columns, Identity::No),
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
