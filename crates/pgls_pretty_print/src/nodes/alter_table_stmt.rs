use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};
use pgls_query::NodeEnum;
use pgls_query::protobuf::{
    AlterTableCmd, AlterTableStmt, AlterTableType, DropBehavior, ObjectType,
};

use super::emit_node;

pub(super) fn emit_alter_table_stmt(e: &mut EventEmitter, n: &AlterTableStmt) {
    e.group_start(GroupKind::AlterTableStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();

    // Emit object type (TABLE, INDEX, TYPE, etc.)
    match n.objtype() {
        ObjectType::ObjectTable => e.token(TokenKind::TABLE_KW),
        ObjectType::ObjectIndex => e.token(TokenKind::INDEX_KW),
        ObjectType::ObjectView => e.token(TokenKind::VIEW_KW),
        ObjectType::ObjectMatview => {
            e.token(TokenKind::MATERIALIZED_KW);
            e.space();
            e.token(TokenKind::VIEW_KW);
        }
        ObjectType::ObjectSequence => e.token(TokenKind::SEQUENCE_KW),
        ObjectType::ObjectForeignTable => {
            e.token(TokenKind::FOREIGN_KW);
            e.space();
            e.token(TokenKind::TABLE_KW);
        }
        ObjectType::ObjectType => e.token(TokenKind::TYPE_KW),
        _ => e.token(TokenKind::TABLE_KW), // Default to TABLE
    }

    e.space();

    if n.missing_ok {
        e.token(TokenKind::IF_KW);
        e.space();
        e.token(TokenKind::EXISTS_KW);
        e.space();
    }

    // Emit relation name
    // For ObjectType (ALTER TYPE), don't emit ONLY since it's not valid
    if let Some(ref relation) = n.relation {
        match n.objtype() {
            ObjectType::ObjectType => super::emit_range_var_name(e, relation),
            _ => super::emit_range_var(e, relation),
        }
    }

    // Emit commands - each on its own indented line
    if !n.cmds.is_empty() {
        e.indent_start();
        e.line(LineType::Hard);

        for (i, cmd_node) in n.cmds.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.line(LineType::Hard);
            }

            // Extract AlterTableCmd from Node
            let cmd = assert_node_variant!(AlterTableCmd, cmd_node);
            let for_type = matches!(n.objtype(), ObjectType::ObjectType);
            emit_alter_table_cmd_for_type(e, cmd, for_type);
        }

        e.indent_end();
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}

/// Emit an ALTER TABLE command. The `for_type` parameter indicates if this is
/// ALTER TYPE (which uses ATTRIBUTE instead of COLUMN).
pub(super) fn emit_alter_table_cmd_for_type(
    e: &mut EventEmitter,
    cmd: &AlterTableCmd,
    for_type: bool,
) {
    emit_alter_table_cmd_impl(e, cmd, for_type);
}

pub(super) fn emit_alter_table_cmd(e: &mut EventEmitter, cmd: &AlterTableCmd) {
    emit_alter_table_cmd_impl(e, cmd, false);
}

fn emit_alter_table_cmd_impl(e: &mut EventEmitter, cmd: &AlterTableCmd, for_type: bool) {
    match cmd.subtype() {
        AlterTableType::AtAddColumn => {
            e.token(TokenKind::ADD_KW);
            e.space();
            // For ALTER TYPE, use ATTRIBUTE instead of COLUMN
            if for_type {
                e.token(TokenKind::ATTRIBUTE_KW);
            } else {
                e.token(TokenKind::COLUMN_KW);
            }
            if let Some(ref def) = cmd.def {
                e.space();
                e.indent_start();
                emit_node(def, e);
                e.indent_end();
            }
        }
        AlterTableType::AtDropColumn => {
            e.token(TokenKind::DROP_KW);
            e.space();
            // For ALTER TYPE, use ATTRIBUTE instead of COLUMN
            if for_type {
                e.token(TokenKind::ATTRIBUTE_KW);
            } else {
                e.token(TokenKind::COLUMN_KW);
            }
            if cmd.missing_ok {
                e.space();
                e.token(TokenKind::IF_KW);
                e.space();
                e.token(TokenKind::EXISTS_KW);
            }
            if !cmd.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(cmd.name.clone()));
            }
            if matches!(cmd.behavior(), DropBehavior::DropCascade) {
                e.space();
                e.token(TokenKind::CASCADE_KW);
            }
        }
        AlterTableType::AtAlterColumnType => {
            e.token(TokenKind::ALTER_KW);
            e.space();
            // For ALTER TYPE, use ATTRIBUTE instead of COLUMN
            if for_type {
                e.token(TokenKind::ATTRIBUTE_KW);
            } else {
                e.token(TokenKind::COLUMN_KW);
            }
            if !cmd.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(cmd.name.clone()));
            }
            e.space();
            e.token(TokenKind::TYPE_KW);
            if let Some(ref def) = cmd.def {
                // Extract ColumnDef from Node to emit only type-related attributes
                let column_def = assert_node_variant!(ColumnDef, def);

                // Emit type name (no space before - TYPE keyword already emitted)
                if let Some(ref typename) = column_def.type_name {
                    e.space();
                    super::emit_type_name(e, typename);
                }

                // Emit compression clause if specified
                if !column_def.compression.is_empty() {
                    e.space();
                    e.token(TokenKind::COMPRESSION_KW);
                    e.space();
                    e.token(TokenKind::IDENT(column_def.compression.clone()));
                }

                // Emit storage clause if specified
                if !column_def.storage_name.is_empty() {
                    e.space();
                    e.token(TokenKind::STORAGE_KW);
                    e.space();
                    e.token(TokenKind::IDENT(column_def.storage_name.clone()));
                }

                // Emit USING clause if specified
                if let Some(ref raw_default) = column_def.raw_default {
                    e.space();
                    e.token(TokenKind::USING_KW);
                    e.space();
                    super::emit_node(raw_default, e);
                }
            }
            // Emit CASCADE/RESTRICT - behavior: 0=Undefined, 1=DropRestrict, 2=DropCascade
            if matches!(cmd.behavior(), DropBehavior::DropCascade) {
                e.space();
                e.token(TokenKind::CASCADE_KW);
            }
        }
        AlterTableType::AtColumnDefault => {
            e.token(TokenKind::ALTER_KW);
            e.space();
            e.token(TokenKind::COLUMN_KW);
            if !cmd.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(cmd.name.clone()));
            }
            e.space();
            if let Some(ref def) = cmd.def {
                e.token(TokenKind::SET_KW);
                e.space();
                e.token(TokenKind::DEFAULT_KW);
                e.space();
                emit_node(def, e);
            } else {
                e.token(TokenKind::DROP_KW);
                e.space();
                e.token(TokenKind::DEFAULT_KW);
            }
        }
        AlterTableType::AtSetNotNull => {
            e.token(TokenKind::ALTER_KW);
            e.space();
            e.token(TokenKind::COLUMN_KW);
            if !cmd.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(cmd.name.clone()));
            }
            e.space();
            e.token(TokenKind::SET_KW);
            e.space();
            e.token(TokenKind::NOT_KW);
            e.space();
            e.token(TokenKind::NULL_KW);
        }
        AlterTableType::AtDropNotNull => {
            e.token(TokenKind::ALTER_KW);
            e.space();
            e.token(TokenKind::COLUMN_KW);
            if !cmd.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(cmd.name.clone()));
            }
            e.space();
            e.token(TokenKind::DROP_KW);
            e.space();
            e.token(TokenKind::NOT_KW);
            e.space();
            e.token(TokenKind::NULL_KW);
        }
        AlterTableType::AtSetExpression => {
            // ALTER COLUMN c SET EXPRESSION AS (expr)
            e.token(TokenKind::ALTER_KW);
            e.space();
            e.token(TokenKind::COLUMN_KW);
            if !cmd.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(cmd.name.clone()));
            }
            e.space();
            e.token(TokenKind::SET_KW);
            e.space();
            e.token(TokenKind::EXPRESSION_KW);
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::AS_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            if let Some(ref def) = cmd.def {
                emit_node(def, e);
            }
            e.token(TokenKind::R_PAREN);
        }
        AlterTableType::AtDropExpression => {
            // ALTER COLUMN c DROP EXPRESSION [IF EXISTS]
            e.token(TokenKind::ALTER_KW);
            e.space();
            e.token(TokenKind::COLUMN_KW);
            if !cmd.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(cmd.name.clone()));
            }
            e.space();
            e.token(TokenKind::DROP_KW);
            e.space();
            e.token(TokenKind::EXPRESSION_KW);
            if cmd.missing_ok {
                e.space();
                e.token(TokenKind::IF_KW);
                e.space();
                e.token(TokenKind::EXISTS_KW);
            }
        }
        AlterTableType::AtAddConstraint => {
            e.token(TokenKind::ADD_KW);
            if let Some(ref def) = cmd.def {
                e.space();
                emit_node(def, e);
            }
        }
        AlterTableType::AtDropConstraint => {
            e.token(TokenKind::DROP_KW);
            e.space();
            e.token(TokenKind::CONSTRAINT_KW);
            if cmd.missing_ok {
                e.space();
                e.token(TokenKind::IF_KW);
                e.space();
                e.token(TokenKind::EXISTS_KW);
            }
            if !cmd.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(cmd.name.clone()));
            }
            // behavior: 0=Undefined, 1=DropRestrict, 2=DropCascade
            if cmd.behavior == 2 {
                e.space();
                e.token(TokenKind::CASCADE_KW);
            }
        }
        AlterTableType::AtValidateConstraint => {
            e.token(TokenKind::VALIDATE_KW);
            e.space();
            e.token(TokenKind::CONSTRAINT_KW);
            if !cmd.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(cmd.name.clone()));
            }
        }
        AlterTableType::AtSetTableSpace => {
            e.token(TokenKind::SET_KW);
            e.space();
            e.token(TokenKind::TABLESPACE_KW);
            if !cmd.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(cmd.name.clone()));
            }
        }
        AlterTableType::AtChangeOwner => {
            e.token(TokenKind::OWNER_KW);
            e.space();
            e.token(TokenKind::TO_KW);
            if let Some(ref owner) = cmd.newowner {
                e.space();
                super::emit_role_spec(e, owner);
            }
        }
        AlterTableType::AtEnableTrig => {
            e.token(TokenKind::ENABLE_KW);
            e.space();
            e.token(TokenKind::TRIGGER_KW);
            if !cmd.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(cmd.name.clone()));
            }
        }
        AlterTableType::AtDisableTrig => {
            e.token(TokenKind::DISABLE_KW);
            e.space();
            e.token(TokenKind::TRIGGER_KW);
            if !cmd.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(cmd.name.clone()));
            }
        }
        AlterTableType::AtSetLogged => {
            e.token(TokenKind::SET_KW);
            e.space();
            e.token(TokenKind::LOGGED_KW);
        }
        AlterTableType::AtSetUnLogged => {
            e.token(TokenKind::SET_KW);
            e.space();
            e.token(TokenKind::UNLOGGED_KW);
        }
        AlterTableType::AtReplicaIdentity => {
            // REPLICA IDENTITY is handled via ReplicaIdentityStmt in the def field
            if let Some(ref def) = cmd.def {
                emit_node(def, e);
            }
        }
        AlterTableType::AtSetRelOptions => {
            e.token(TokenKind::SET_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            if let Some(ref def) = cmd.def {
                emit_node(def, e); // List of DefElem nodes
            }
            e.token(TokenKind::R_PAREN);
        }
        AlterTableType::AtResetRelOptions => {
            e.token(TokenKind::RESET_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            if let Some(ref def) = cmd.def {
                emit_node(def, e); // List of option names
            }
            e.token(TokenKind::R_PAREN);
        }
        AlterTableType::AtSetOptions => {
            // ALTER COLUMN col SET (options)
            e.token(TokenKind::ALTER_KW);
            e.space();
            e.token(TokenKind::COLUMN_KW);
            if !cmd.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(cmd.name.clone()));
            }
            e.space();
            e.token(TokenKind::SET_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            if let Some(ref def) = cmd.def {
                emit_node(def, e); // List of DefElem nodes
            }
            e.token(TokenKind::R_PAREN);
        }
        AlterTableType::AtResetOptions => {
            // ALTER COLUMN col RESET (options)
            e.token(TokenKind::ALTER_KW);
            e.space();
            e.token(TokenKind::COLUMN_KW);
            if !cmd.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(cmd.name.clone()));
            }
            e.space();
            e.token(TokenKind::RESET_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            if let Some(ref def) = cmd.def {
                emit_node(def, e); // List of option names
            }
            e.token(TokenKind::R_PAREN);
        }
        AlterTableType::AtSetStatistics => {
            // ALTER COLUMN col SET STATISTICS value
            e.token(TokenKind::ALTER_KW);
            e.space();
            e.token(TokenKind::COLUMN_KW);
            if !cmd.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(cmd.name.clone()));
            } else if cmd.num != 0 {
                // Column specified by number (for indexes)
                e.space();
                e.token(TokenKind::IDENT(cmd.num.to_string()));
            }
            e.space();
            e.token(TokenKind::SET_KW);
            e.space();
            e.token(TokenKind::STATISTICS_KW);
            if let Some(ref def) = cmd.def {
                e.space();
                emit_node(def, e);
            }
        }
        AlterTableType::AtSetStorage => {
            // ALTER COLUMN col SET STORAGE {PLAIN|EXTERNAL|EXTENDED|MAIN}
            e.token(TokenKind::ALTER_KW);
            e.space();
            e.token(TokenKind::COLUMN_KW);
            if !cmd.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(cmd.name.clone()));
            }
            e.space();
            e.token(TokenKind::SET_KW);
            e.space();
            e.token(TokenKind::STORAGE_KW);
            if let Some(ref def) = cmd.def {
                e.space();
                emit_node(def, e);
            }
        }
        AlterTableType::AtSetCompression => {
            // ALTER COLUMN col SET COMPRESSION method
            e.token(TokenKind::ALTER_KW);
            e.space();
            e.token(TokenKind::COLUMN_KW);
            if !cmd.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(cmd.name.clone()));
            }
            e.space();
            e.token(TokenKind::SET_KW);
            e.space();
            e.token(TokenKind::COMPRESSION_KW);
            if let Some(ref def) = cmd.def {
                e.space();
                emit_node(def, e);
            }
        }
        AlterTableType::AtClusterOn => {
            e.token(TokenKind::CLUSTER_KW);
            e.space();
            e.token(TokenKind::ON_KW);
            if !cmd.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(cmd.name.clone()));
            }
        }
        AlterTableType::AtDropCluster => {
            e.token(TokenKind::SET_KW);
            e.space();
            e.token(TokenKind::WITHOUT_KW);
            e.space();
            e.token(TokenKind::CLUSTER_KW);
        }
        AlterTableType::AtSetAccessMethod => {
            e.token(TokenKind::SET_KW);
            e.space();
            e.token(TokenKind::ACCESS_KW);
            e.space();
            e.token(TokenKind::METHOD_KW);
            e.space();
            if !cmd.name.is_empty() {
                e.token(TokenKind::IDENT(cmd.name.clone()));
            } else {
                // Empty name means DEFAULT
                e.token(TokenKind::DEFAULT_KW);
            }
        }
        AlterTableType::AtEnableRowSecurity => {
            e.token(TokenKind::ENABLE_KW);
            e.space();
            e.token(TokenKind::ROW_KW);
            e.space();
            e.token(TokenKind::LEVEL_KW);
            e.space();
            e.token(TokenKind::SECURITY_KW);
        }
        AlterTableType::AtDisableRowSecurity => {
            e.token(TokenKind::DISABLE_KW);
            e.space();
            e.token(TokenKind::ROW_KW);
            e.space();
            e.token(TokenKind::LEVEL_KW);
            e.space();
            e.token(TokenKind::SECURITY_KW);
        }
        AlterTableType::AtForceRowSecurity => {
            e.token(TokenKind::FORCE_KW);
            e.space();
            e.token(TokenKind::ROW_KW);
            e.space();
            e.token(TokenKind::LEVEL_KW);
            e.space();
            e.token(TokenKind::SECURITY_KW);
        }
        AlterTableType::AtNoForceRowSecurity => {
            e.token(TokenKind::NO_KW);
            e.space();
            e.token(TokenKind::FORCE_KW);
            e.space();
            e.token(TokenKind::ROW_KW);
            e.space();
            e.token(TokenKind::LEVEL_KW);
            e.space();
            e.token(TokenKind::SECURITY_KW);
        }
        AlterTableType::AtAddInherit => {
            e.token(TokenKind::INHERIT_KW);
            if let Some(ref def) = cmd.def {
                e.space();
                emit_node(def, e);
            }
        }
        AlterTableType::AtDropInherit => {
            e.token(TokenKind::NO_KW);
            e.space();
            e.token(TokenKind::INHERIT_KW);
            if let Some(ref def) = cmd.def {
                e.space();
                emit_node(def, e);
            }
        }
        AlterTableType::AtAddOf => {
            e.token(TokenKind::OF_KW);
            if let Some(ref def) = cmd.def {
                e.space();
                emit_node(def, e);
            }
        }
        AlterTableType::AtDropOf => {
            e.token(TokenKind::NOT_KW);
            e.space();
            e.token(TokenKind::OF_KW);
        }
        AlterTableType::AtAttachPartition => {
            e.token(TokenKind::ATTACH_KW);
            e.space();
            e.token(TokenKind::PARTITION_KW);
            if let Some(ref def) = cmd.def {
                e.line(LineType::SoftOrSpace);
                emit_node(def, e); // PartitionCmd node
            }
        }
        AlterTableType::AtDetachPartition => {
            e.token(TokenKind::DETACH_KW);
            e.space();
            e.token(TokenKind::PARTITION_KW);
            if let Some(ref def) = cmd.def {
                e.line(LineType::SoftOrSpace);
                emit_node(def, e); // PartitionCmd node
            }
        }
        AlterTableType::AtEnableTrigAll => {
            e.token(TokenKind::ENABLE_KW);
            e.space();
            e.token(TokenKind::TRIGGER_KW);
            e.space();
            e.token(TokenKind::ALL_KW);
        }
        AlterTableType::AtDisableTrigAll => {
            e.token(TokenKind::DISABLE_KW);
            e.space();
            e.token(TokenKind::TRIGGER_KW);
            e.space();
            e.token(TokenKind::ALL_KW);
        }
        AlterTableType::AtEnableTrigUser => {
            e.token(TokenKind::ENABLE_KW);
            e.space();
            e.token(TokenKind::TRIGGER_KW);
            e.space();
            e.token(TokenKind::USER_KW);
        }
        AlterTableType::AtDisableTrigUser => {
            e.token(TokenKind::DISABLE_KW);
            e.space();
            e.token(TokenKind::TRIGGER_KW);
            e.space();
            e.token(TokenKind::USER_KW);
        }
        AlterTableType::AtEnableAlwaysTrig => {
            e.token(TokenKind::ENABLE_KW);
            e.space();
            e.token(TokenKind::ALWAYS_KW);
            e.space();
            e.token(TokenKind::TRIGGER_KW);
            if !cmd.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(cmd.name.clone()));
            }
        }
        AlterTableType::AtEnableReplicaTrig => {
            e.token(TokenKind::ENABLE_KW);
            e.space();
            e.token(TokenKind::REPLICA_KW);
            e.space();
            e.token(TokenKind::TRIGGER_KW);
            if !cmd.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(cmd.name.clone()));
            }
        }
        AlterTableType::AtEnableRule => {
            e.token(TokenKind::ENABLE_KW);
            e.space();
            e.token(TokenKind::RULE_KW);
            if !cmd.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(cmd.name.clone()));
            }
        }
        AlterTableType::AtDisableRule => {
            e.token(TokenKind::DISABLE_KW);
            e.space();
            e.token(TokenKind::RULE_KW);
            if !cmd.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(cmd.name.clone()));
            }
        }
        AlterTableType::AtEnableAlwaysRule => {
            e.token(TokenKind::ENABLE_KW);
            e.space();
            e.token(TokenKind::ALWAYS_KW);
            e.space();
            e.token(TokenKind::RULE_KW);
            if !cmd.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(cmd.name.clone()));
            }
        }
        AlterTableType::AtEnableReplicaRule => {
            e.token(TokenKind::ENABLE_KW);
            e.space();
            e.token(TokenKind::REPLICA_KW);
            e.space();
            e.token(TokenKind::RULE_KW);
            if !cmd.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(cmd.name.clone()));
            }
        }
        AlterTableType::AtAddIdentity => {
            // ALTER COLUMN col ADD GENERATED ALWAYS/BY DEFAULT AS IDENTITY (options)
            // def is a Constraint node with identity options
            e.token(TokenKind::ALTER_KW);
            e.space();
            e.token(TokenKind::COLUMN_KW);
            if !cmd.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(cmd.name.clone()));
            }
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::ADD_KW);
            e.space();
            // The def is a Constraint containing the full identity definition
            // Just emit the constraint which handles GENERATED ALWAYS/BY DEFAULT AS IDENTITY
            if let Some(ref def) = cmd.def {
                emit_node(def, e);
            }
        }
        AlterTableType::AtSetIdentity => {
            // ALTER COLUMN col SET GENERATED BY DEFAULT/ALWAYS or SET seq options
            e.token(TokenKind::ALTER_KW);
            e.space();
            e.token(TokenKind::COLUMN_KW);
            if !cmd.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(cmd.name.clone()));
            }
            e.space();
            e.token(TokenKind::SET_KW);
            // def is a List of DefElem options
            if let Some(ref def) = cmd.def {
                if let Some(NodeEnum::List(list)) = &def.node {
                    for opt in &list.items {
                        if let Some(NodeEnum::DefElem(de)) = &opt.node {
                            e.space();
                            // Handle special "generated" option that specifies ALWAYS/BY DEFAULT
                            if de.defname == "generated" {
                                e.token(TokenKind::GENERATED_KW);
                                e.space();
                                // arg is an Integer - 97='a' for ALWAYS, 100='d' for BY DEFAULT
                                if let Some(ref arg) = de.arg {
                                    if let Some(NodeEnum::Integer(i)) = &arg.node {
                                        if i.ival == 97 {
                                            // 'a' = ALWAYS
                                            e.token(TokenKind::ALWAYS_KW);
                                        } else {
                                            // 'd' = BY DEFAULT
                                            e.token(TokenKind::BY_KW);
                                            e.space();
                                            e.token(TokenKind::DEFAULT_KW);
                                        }
                                    }
                                }
                            } else {
                                // Emit other sequence options
                                super::emit_sequence_option(e, de);
                            }
                        }
                    }
                }
            }
        }
        AlterTableType::AtDropIdentity => {
            // ALTER COLUMN col DROP IDENTITY [IF EXISTS]
            e.token(TokenKind::ALTER_KW);
            e.space();
            e.token(TokenKind::COLUMN_KW);
            if !cmd.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(cmd.name.clone()));
            }
            e.space();
            e.token(TokenKind::DROP_KW);
            e.space();
            e.token(TokenKind::IDENTITY_KW);
            if cmd.missing_ok {
                e.space();
                e.token(TokenKind::IF_KW);
                e.space();
                e.token(TokenKind::EXISTS_KW);
            }
        }
        AlterTableType::AtAlterConstraint => {
            // ALTER CONSTRAINT constraint_name [DEFERRABLE | NOT DEFERRABLE] [INITIALLY DEFERRED | INITIALLY IMMEDIATE]
            e.token(TokenKind::ALTER_KW);
            e.space();
            e.token(TokenKind::CONSTRAINT_KW);
            e.space();

            // constraint name comes from the def (Constraint node)
            if let Some(ref def) = cmd.def {
                if let Some(NodeEnum::Constraint(c)) = &def.node {
                    super::string::emit_identifier(e, &c.conname);

                    // DEFERRABLE or NOT DEFERRABLE
                    if c.deferrable {
                        e.space();
                        e.token(TokenKind::DEFERRABLE_KW);
                    } else {
                        e.space();
                        e.token(TokenKind::NOT_KW);
                        e.space();
                        e.token(TokenKind::DEFERRABLE_KW);
                    }

                    // INITIALLY DEFERRED or INITIALLY IMMEDIATE
                    if c.initdeferred {
                        e.space();
                        e.token(TokenKind::INITIALLY_KW);
                        e.space();
                        e.token(TokenKind::DEFERRED_KW);
                    } else {
                        e.space();
                        e.token(TokenKind::INITIALLY_KW);
                        e.space();
                        e.token(TokenKind::IMMEDIATE_KW);
                    }
                }
            }
        }
        AlterTableType::AtDropOids => {
            // ALTER TABLE ... SET WITHOUT OIDS (deprecated but still parsed)
            e.token(TokenKind::SET_KW);
            e.space();
            e.token(TokenKind::WITHOUT_KW);
            e.space();
            e.token(TokenKind::OIDS_KW);
        }
        AlterTableType::AtAlterColumnGenericOptions => {
            // ALTER COLUMN col OPTIONS (SET/ADD/DROP name 'value')
            e.token(TokenKind::ALTER_KW);
            e.space();
            e.token(TokenKind::COLUMN_KW);
            if !cmd.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(cmd.name.clone()));
            }
            e.space();
            e.token(TokenKind::OPTIONS_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            if let Some(ref def) = cmd.def {
                if let Some(NodeEnum::List(list)) = &def.node {
                    for (i, opt) in list.items.iter().enumerate() {
                        if i > 0 {
                            e.token(TokenKind::COMMA);
                            e.space();
                        }
                        if let Some(NodeEnum::DefElem(de)) = &opt.node {
                            super::emit_options_def_elem(e, de);
                        }
                    }
                }
            }
            e.token(TokenKind::R_PAREN);
        }
        AlterTableType::AtGenericOptions => {
            // OPTIONS (SET/ADD/DROP name 'value') - table-level options
            e.token(TokenKind::OPTIONS_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            if let Some(ref def) = cmd.def {
                if let Some(NodeEnum::List(list)) = &def.node {
                    for (i, opt) in list.items.iter().enumerate() {
                        if i > 0 {
                            e.token(TokenKind::COMMA);
                            e.space();
                        }
                        if let Some(NodeEnum::DefElem(de)) = &opt.node {
                            super::emit_options_def_elem(e, de);
                        }
                    }
                }
            }
            e.token(TokenKind::R_PAREN);
        }
        _ => {
            // Fallback for unimplemented subtypes
            debug_assert!(false, "Unhandled AlterTableType: {:?}", cmd.subtype());
            e.token(TokenKind::IDENT(format!("TODO: {:?}", cmd.subtype())));
        }
    }
}
