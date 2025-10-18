use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgt_query::protobuf::{AlterTableCmd, AlterTableStmt, AlterTableType, ObjectType};

use super::emit_node;

pub(super) fn emit_alter_table_stmt(e: &mut EventEmitter, n: &AlterTableStmt) {
    e.group_start(GroupKind::AlterTableStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();

    // Emit object type (TABLE, INDEX, etc.)
    let object_type = ObjectType::try_from(n.objtype).unwrap_or(ObjectType::Undefined);
    match object_type {
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
    if let Some(ref relation) = n.relation {
        super::emit_range_var(e, relation);
    }

    // Emit commands
    if !n.cmds.is_empty() {
        for (i, cmd_node) in n.cmds.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
            }
            e.space();

            // Extract AlterTableCmd from Node
            let cmd = assert_node_variant!(AlterTableCmd, cmd_node);
            emit_alter_table_cmd(e, cmd);
        }
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}

fn emit_alter_table_cmd(e: &mut EventEmitter, cmd: &AlterTableCmd) {
    let subtype = AlterTableType::try_from(cmd.subtype).unwrap_or(AlterTableType::Undefined);

    match subtype {
        AlterTableType::AtAddColumn => {
            e.token(TokenKind::ADD_KW);
            e.space();
            e.token(TokenKind::COLUMN_KW);
            if let Some(ref def) = cmd.def {
                e.space();
                emit_node(def, e);
            }
        }
        AlterTableType::AtDropColumn => {
            e.token(TokenKind::DROP_KW);
            e.space();
            e.token(TokenKind::COLUMN_KW);
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
        AlterTableType::AtAlterColumnType => {
            e.token(TokenKind::ALTER_KW);
            e.space();
            e.token(TokenKind::COLUMN_KW);
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
            e.token(TokenKind::IDENT("TABLESPACE".to_string()));
            if !cmd.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(cmd.name.clone()));
            }
        }
        AlterTableType::AtChangeOwner => {
            e.token(TokenKind::IDENT("OWNER".to_string()));
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
            e.token(TokenKind::IDENT("LOGGED".to_string()));
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
            e.token(TokenKind::IDENT("RESET".to_string()));
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
            e.token(TokenKind::IDENT("RESET".to_string()));
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
            }
            e.space();
            e.token(TokenKind::SET_KW);
            e.space();
            e.token(TokenKind::IDENT("STATISTICS".to_string()));
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
            e.token(TokenKind::IDENT("STORAGE".to_string()));
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
            e.token(TokenKind::IDENT("COMPRESSION".to_string()));
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
            e.token(TokenKind::IDENT("ACCESS".to_string()));
            e.space();
            e.token(TokenKind::IDENT("METHOD".to_string()));
            if !cmd.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(cmd.name.clone()));
            }
        }
        AlterTableType::AtEnableRowSecurity => {
            e.token(TokenKind::ENABLE_KW);
            e.space();
            e.token(TokenKind::IDENT("ROW".to_string()));
            e.space();
            e.token(TokenKind::IDENT("LEVEL".to_string()));
            e.space();
            e.token(TokenKind::IDENT("SECURITY".to_string()));
        }
        AlterTableType::AtDisableRowSecurity => {
            e.token(TokenKind::DISABLE_KW);
            e.space();
            e.token(TokenKind::IDENT("ROW".to_string()));
            e.space();
            e.token(TokenKind::IDENT("LEVEL".to_string()));
            e.space();
            e.token(TokenKind::IDENT("SECURITY".to_string()));
        }
        AlterTableType::AtForceRowSecurity => {
            e.token(TokenKind::IDENT("FORCE".to_string()));
            e.space();
            e.token(TokenKind::IDENT("ROW".to_string()));
            e.space();
            e.token(TokenKind::IDENT("LEVEL".to_string()));
            e.space();
            e.token(TokenKind::IDENT("SECURITY".to_string()));
        }
        AlterTableType::AtNoForceRowSecurity => {
            e.token(TokenKind::IDENT("NO".to_string()));
            e.space();
            e.token(TokenKind::IDENT("FORCE".to_string()));
            e.space();
            e.token(TokenKind::IDENT("ROW".to_string()));
            e.space();
            e.token(TokenKind::IDENT("LEVEL".to_string()));
            e.space();
            e.token(TokenKind::IDENT("SECURITY".to_string()));
        }
        AlterTableType::AtAddInherit => {
            e.token(TokenKind::IDENT("INHERIT".to_string()));
            if let Some(ref def) = cmd.def {
                e.space();
                emit_node(def, e);
            }
        }
        AlterTableType::AtDropInherit => {
            e.token(TokenKind::IDENT("NO".to_string()));
            e.space();
            e.token(TokenKind::IDENT("INHERIT".to_string()));
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
            e.token(TokenKind::IDENT("ATTACH".to_string()));
            e.space();
            e.token(TokenKind::PARTITION_KW);
            if let Some(ref def) = cmd.def {
                e.space();
                emit_node(def, e); // PartitionCmd node
            }
        }
        AlterTableType::AtDetachPartition => {
            e.token(TokenKind::IDENT("DETACH".to_string()));
            e.space();
            e.token(TokenKind::PARTITION_KW);
            if let Some(ref def) = cmd.def {
                e.space();
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
            e.token(TokenKind::IDENT("ALWAYS".to_string()));
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
            e.token(TokenKind::IDENT("REPLICA".to_string()));
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
            e.token(TokenKind::IDENT("RULE".to_string()));
            if !cmd.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(cmd.name.clone()));
            }
        }
        AlterTableType::AtDisableRule => {
            e.token(TokenKind::DISABLE_KW);
            e.space();
            e.token(TokenKind::IDENT("RULE".to_string()));
            if !cmd.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(cmd.name.clone()));
            }
        }
        AlterTableType::AtEnableAlwaysRule => {
            e.token(TokenKind::ENABLE_KW);
            e.space();
            e.token(TokenKind::IDENT("ALWAYS".to_string()));
            e.space();
            e.token(TokenKind::IDENT("RULE".to_string()));
            if !cmd.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(cmd.name.clone()));
            }
        }
        AlterTableType::AtEnableReplicaRule => {
            e.token(TokenKind::ENABLE_KW);
            e.space();
            e.token(TokenKind::IDENT("REPLICA".to_string()));
            e.space();
            e.token(TokenKind::IDENT("RULE".to_string()));
            if !cmd.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(cmd.name.clone()));
            }
        }
        AlterTableType::AtAddIdentity => {
            // ALTER COLUMN col ADD IDENTITY
            e.token(TokenKind::ALTER_KW);
            e.space();
            e.token(TokenKind::COLUMN_KW);
            if !cmd.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(cmd.name.clone()));
            }
            e.space();
            e.token(TokenKind::ADD_KW);
            e.space();
            e.token(TokenKind::IDENT("GENERATED".to_string()));
            e.space();
            e.token(TokenKind::IDENT("ALWAYS".to_string()));
            e.space();
            e.token(TokenKind::AS_KW);
            e.space();
            e.token(TokenKind::IDENT("IDENTITY".to_string()));
            if let Some(ref def) = cmd.def {
                e.space();
                emit_node(def, e);
            }
        }
        AlterTableType::AtSetIdentity => {
            // ALTER COLUMN col SET seq options
            e.token(TokenKind::ALTER_KW);
            e.space();
            e.token(TokenKind::COLUMN_KW);
            if !cmd.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(cmd.name.clone()));
            }
            e.space();
            e.token(TokenKind::SET_KW);
            if let Some(ref def) = cmd.def {
                e.space();
                emit_node(def, e);
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
            e.token(TokenKind::IDENT("IDENTITY".to_string()));
            if cmd.missing_ok {
                e.space();
                e.token(TokenKind::IF_KW);
                e.space();
                e.token(TokenKind::EXISTS_KW);
            }
        }
        _ => {
            // Fallback for unimplemented subtypes
            e.token(TokenKind::IDENT(format!("TODO: {:?}", subtype)));
        }
    }
}
