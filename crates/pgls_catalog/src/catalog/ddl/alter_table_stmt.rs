use pgls_query::{
    NodeEnum,
    protobuf::{AlterTableCmd, AlterTableStmt, AlterTableType, ObjectType},
};
use pgls_query_ext::options::find_def_elem;
use pgls_query_ext::utils::{alter_table_cmds, cmd_def};

use crate::catalog::{
    Catalog, Key, key,
    names::{range_var_name, type_label},
    overlay::column_info,
};
use crate::lookup::{ColumnInfo, Origin, RelationInfo, RelationKind};
use crate::{Generated, Identity};

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
    let commands: Vec<&AlterTableCmd> = alter_table_cmds(n).collect();

    if n.objtype() == ObjectType::ObjectType {
        c.change_type(&range_var_name(range_var), search_path, |c, type_info| {
            if let Some(attributes) = type_info.attributes.as_mut() {
                for command in &commands {
                    apply_column_change(c, attributes, command, search_path, IdentityChange::Apply);
                }
            }
        });
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
    apply_index_changes(c, &relation, &commands);
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
    if changes_inherited && c.is_inheritance_child(relation) {
        c.change_columns(relation, |columns, _| *columns = None);
        return;
    }
    // `ADD/SET/DROP IDENTITY` recurse to the partitions of a partitioned table on Postgres 17+
    // ([`ATExecAddIdentity`], [`ATExecSetIdentity`], [`ATExecDropIdentity`]), and to no child
    // before ([`ATPrepCmd`]). Inheritance children never follow them, and keep their own
    // identity. Only partitions on Postgres 17+ get the identity of an added column
    // ([`ATPrepAddColumn`]).
    //
    // [`ATExecAddIdentity`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/tablecmds.c#L8354
    // [`ATExecSetIdentity`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/tablecmds.c#L8471
    // [`ATExecDropIdentity`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/tablecmds.c#L8572
    // [`ATPrepCmd`]: https://github.com/postgres/postgres/blob/REL_15_14/src/backend/commands/tablecmds.c#L4605
    // [`ATPrepAddColumn`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/tablecmds.c#L7352
    let descendant_identity = if relation.kind == RelationKind::PartitionedTable {
        match c.partitions_inherit_identity() {
            Some(true) => IdentityChange::Apply,
            Some(false) => IdentityChange::Keep,
            None => IdentityChange::Unknown,
        }
    } else {
        IdentityChange::Keep
    };
    let catalog = c.clone();
    c.change_columns(relation, |columns, is_descendant| {
        if let Some(columns) = columns.as_mut() {
            let identity = if is_descendant {
                descendant_identity
            } else {
                IdentityChange::Apply
            };
            for command in commands {
                apply_column_change(&catalog, columns, command, search_path, identity);
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
        let children = c.children.entry(parent.clone()).or_default();
        if attach {
            if !children.contains(&child) {
                children.push(child.clone());
            }
        } else {
            children.retain(|existing| *existing != child);
        }
        if relation_is_parent {
            if attach {
                attach_partition_identity(c, &parent, &child);
            } else {
                detach_partition_identity(c, &child);
            }
        }
    }
}

/// On Postgres 17+, an attached partition takes the identity of its partitioned table's columns
/// ([`MergeAttributesIntoExisting`]); its own partitions keep theirs. Before, it keeps its own.
///
/// [`MergeAttributesIntoExisting`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/tablecmds.c#L17619
fn attach_partition_identity(c: &mut Catalog, parent: &Key, partition: &Key) {
    let inherits_identity = c.partitions_inherit_identity();
    if inherits_identity == Some(false) {
        return;
    }
    let Some(mut info) = c.relation_in(&partition.0, &partition.1).found() else {
        return;
    };
    let parent_columns = c
        .relation_in(&parent.0, &parent.1)
        .found()
        .and_then(|parent| parent.columns);
    let Some(columns) = info.columns.as_mut() else {
        return;
    };
    for column in columns {
        let parent_identity = parent_columns
            .iter()
            .flatten()
            .find(|parent| parent.name == column.name)
            .map(|parent| parent.identity);
        column.identity = match (inherits_identity, parent_identity) {
            (Some(true), Some(identity)) => identity,
            // Without the server version, the identity is only known if both versions agree.
            (None, Some(identity)) if identity == column.identity => identity,
            _ => Identity::No,
        };
    }
    info.origin = Origin::File;
    c.define_relation(info);
}

/// On Postgres 17+, `DETACH PARTITION` drops the identity of the partition's identity columns,
/// and of the same columns of its own partitions ([`DetachPartitionFinalize`]). Before, the
/// partition keeps its identity.
///
/// [`DetachPartitionFinalize`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/tablecmds.c#L21416
fn detach_partition_identity(c: &mut Catalog, partition: &Key) {
    if c.partitions_inherit_identity() == Some(false) {
        return;
    }
    let Some(info) = c.relation_in(&partition.0, &partition.1).found() else {
        return;
    };
    // Without the partition's columns, the identity of every column is unknown.
    let dropped: Option<Vec<String>> = info.columns.as_ref().map(|columns| {
        columns
            .iter()
            .filter(|column| column.identity != Identity::No)
            .map(|column| column.name.clone())
            .collect()
    });
    if dropped.as_ref().is_some_and(Vec::is_empty) {
        return;
    }
    c.change_columns(&info, |columns, _| {
        for column in columns.iter_mut().flatten() {
            if dropped
                .as_ref()
                .is_none_or(|dropped| dropped.contains(&column.name))
            {
                column.identity = Identity::No;
            }
        }
    });
}

/// What an identity change does to a column of a table or of one of its descendants.
#[derive(Clone, Copy, PartialEq, Eq)]
enum IdentityChange {
    /// The column takes the new identity.
    Apply,
    /// The column keeps its identity.
    Keep,
    /// The column's identity is unknown, so it's not treated as an identity column.
    Unknown,
}

/// The changes of `ALTER TABLE` to the indexes of the table:
/// - `ADD CONSTRAINT ... USING INDEX` renames the index, which must be on the table and in its
///   schema, to the name of the constraint ([`transformIndexConstraint`],
///   [`ATExecAddIndexConstraint`]).
/// - `DROP CONSTRAINT` drops the index of a primary key, unique or exclusion constraint, and on a
///   partitioned table also the constraints and indexes of the partitions
///   ([`dropconstraint_internal`]). We don't know which constraints have an index, so the index
///   with the name of the constraint becomes unknown.
///
/// [`transformIndexConstraint`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_utilcmd.c#L2406
/// [`ATExecAddIndexConstraint`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/tablecmds.c#L9715
/// [`dropconstraint_internal`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/tablecmds.c#L14098
fn apply_index_changes(c: &mut Catalog, table: &RelationInfo, commands: &[&AlterTableCmd]) {
    for command in commands {
        match command.subtype() {
            AlterTableType::AtAddConstraint => {
                let Some(NodeEnum::Constraint(constraint)) =
                    command.def.as_deref().and_then(|def| def.node.as_ref())
                else {
                    continue;
                };
                if constraint.indexname.is_empty() || constraint.conname.is_empty() {
                    continue;
                }
                // Postgres rejects an index that is missing or on another table. If the index
                // is unknown, so is the new name, see `creates_implicit_indexes`.
                if let Some(index) = c
                    .index_in(&table.schema, &constraint.indexname)
                    .found()
                    .filter(|index| index.table == table.name)
                {
                    c.move_index(index, &constraint.conname);
                }
            }
            AlterTableType::AtDropConstraint => {
                c.mark_index_unknown(&table.schema, &command.name);
                if table.kind == RelationKind::PartitionedTable {
                    c.drop_partition_indexes();
                }
            }
            _ => {}
        }
    }
}

/// Applies `ADD COLUMN`, `DROP COLUMN` and `ALTER COLUMN TYPE` (and their `ATTRIBUTE`
/// counterparts), `ALTER COLUMN DROP EXPRESSION`, which turns a generated column into an
/// ordinary one, and the identity changes `ADD GENERATED ... AS IDENTITY`, `SET GENERATED` and
/// `DROP IDENTITY`, as `identity_change` says. A column added to a table that doesn't
/// [`IdentityChange::Apply`] the identity changes isn't an identity column.
fn apply_column_change(
    c: &Catalog,
    columns: &mut Vec<ColumnInfo>,
    command: &AlterTableCmd,
    search_path: &[String],
    identity_change: IdentityChange,
) {
    let set_identity = |columns: &mut Vec<ColumnInfo>, identity: Identity| {
        if let Some(column) = columns
            .iter_mut()
            .find(|column| column.name == command.name)
        {
            match identity_change {
                IdentityChange::Apply => column.identity = identity,
                IdentityChange::Keep => {}
                IdentityChange::Unknown => column.identity = Identity::No,
            }
        }
    };
    match command.subtype() {
        AlterTableType::AtAddColumn => {
            if let Some(NodeEnum::ColumnDef(column)) = cmd_def(command)
                && !columns.iter().any(|c| c.name == column.colname)
            {
                let mut column = column_info(c, column, search_path);
                if identity_change != IdentityChange::Apply {
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
                set_identity(columns, Identity::from_code(&constraint.generated_when));
            }
        }
        // `SET GENERATED { ALWAYS | BY DEFAULT }` among the sequence options.
        AlterTableType::AtSetIdentity => {
            let generated = match command.def.as_deref().and_then(|def| def.node.as_ref()) {
                Some(NodeEnum::List(options)) => find_def_elem(&options.items, "generated")
                    .and_then(|option| match option.arg.as_deref()?.node.as_ref()? {
                        NodeEnum::Integer(value) => u8::try_from(value.ival).ok(),
                        _ => None,
                    }),
                _ => None,
            };
            if let Some(generated) = generated {
                set_identity(
                    columns,
                    Identity::from_code(&char::from(generated).to_string()),
                );
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
                cmd_def(command),
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
