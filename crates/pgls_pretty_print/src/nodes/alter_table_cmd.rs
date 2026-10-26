use pgls_query::protobuf::{AlterTableCmd, AlterTableType, ObjectType};

use super::prelude::*;

pub(super) fn emit_alter_table_cmd(e: &mut EventEmitter, cmd: &AlterTableCmd) {
    emit_alter_table_cmd_of(e, cmd, ObjectType::ObjectTable);
}

/// Emits a command of `ALTER <object_type>`. ALTER TYPE writes ATTRIBUTE where the other
/// statements write COLUMN.
pub(super) fn emit_alter_table_cmd_of(
    e: &mut EventEmitter,
    cmd: &AlterTableCmd,
    object_type: ObjectType,
) {
    let for_type = object_type == ObjectType::ObjectType;
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
            // Dropping the flag turns an idempotent DDL into one that fails on a second run, so
            // the AST round trip guard refuses the whole statement rather than let it through.
            if cmd.missing_ok {
                e.space();
                emit_if_not_exists(e);
            }
            if let Some(ref def) = cmd.def {
                e.space();
                e.indent_start();
                emit_node(e, def);
                e.indent_end();
            }
            emit_drop_behavior(e, cmd.behavior(), Separator::Space, Restrict::Omitted);
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
                emit_if_exists(e);
            }
            if !cmd.name.is_empty() {
                e.space();
                emit_identifier_maybe_quoted(e, &cmd.name);
            }
            emit_drop_behavior(e, cmd.behavior(), Separator::Space, Restrict::Omitted);
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
                emit_identifier_maybe_quoted(e, &cmd.name);
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

                if let Some(ref coll_clause) = column_def.coll_clause {
                    e.space();
                    super::emit_collate_clause(e, coll_clause);
                }

                // Emit compression clause if specified
                if !column_def.compression.is_empty() {
                    e.space();
                    e.token(TokenKind::COMPRESSION_KW);
                    e.space();
                    emit_identifier_maybe_quoted(e, &column_def.compression);
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
                    emit_node(e, raw_default);
                }
            }
            emit_drop_behavior(e, cmd.behavior(), Separator::Space, Restrict::Omitted);
        }
        AlterTableType::AtColumnDefault => {
            e.token(TokenKind::ALTER_KW);
            e.space();
            e.token(TokenKind::COLUMN_KW);
            if !cmd.name.is_empty() {
                e.space();
                emit_identifier_maybe_quoted(e, &cmd.name);
            }
            e.space();
            if let Some(ref def) = cmd.def {
                e.token(TokenKind::SET_KW);
                e.space();
                e.token(TokenKind::DEFAULT_KW);
                e.space();
                emit_node(e, def);
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
                emit_identifier_maybe_quoted(e, &cmd.name);
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
                emit_identifier_maybe_quoted(e, &cmd.name);
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
                emit_identifier_maybe_quoted(e, &cmd.name);
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
                emit_node(e, def);
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
                emit_identifier_maybe_quoted(e, &cmd.name);
            }
            e.space();
            e.token(TokenKind::DROP_KW);
            e.space();
            e.token(TokenKind::EXPRESSION_KW);
            if cmd.missing_ok {
                e.space();
                emit_if_exists(e);
            }
        }
        AlterTableType::AtAddConstraint => {
            e.token(TokenKind::ADD_KW);
            if let Some(ref def) = cmd.def {
                e.space();
                emit_node(e, def);
            }
        }
        AlterTableType::AtDropConstraint => {
            e.token(TokenKind::DROP_KW);
            e.space();
            e.token(TokenKind::CONSTRAINT_KW);
            if cmd.missing_ok {
                e.space();
                emit_if_exists(e);
            }
            if !cmd.name.is_empty() {
                e.space();
                emit_identifier_maybe_quoted(e, &cmd.name);
            }
            emit_drop_behavior(e, cmd.behavior(), Separator::Space, Restrict::Omitted);
        }
        AlterTableType::AtValidateConstraint => {
            e.token(TokenKind::VALIDATE_KW);
            e.space();
            e.token(TokenKind::CONSTRAINT_KW);
            if !cmd.name.is_empty() {
                e.space();
                emit_identifier_maybe_quoted(e, &cmd.name);
            }
        }
        AlterTableType::AtSetTableSpace => {
            e.token(TokenKind::SET_KW);
            e.space();
            e.token(TokenKind::TABLESPACE_KW);
            if !cmd.name.is_empty() {
                e.space();
                emit_identifier_maybe_quoted(e, &cmd.name);
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
                emit_identifier_maybe_quoted(e, &cmd.name);
            }
        }
        AlterTableType::AtDisableTrig => {
            e.token(TokenKind::DISABLE_KW);
            e.space();
            e.token(TokenKind::TRIGGER_KW);
            if !cmd.name.is_empty() {
                e.space();
                emit_identifier_maybe_quoted(e, &cmd.name);
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
                emit_node(e, def);
            }
        }
        AlterTableType::AtSetRelOptions => {
            e.token(TokenKind::SET_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            if let Some(ref def) = cmd.def {
                emit_node(e, def); // List of DefElem nodes
            }
            e.token(TokenKind::R_PAREN);
        }
        AlterTableType::AtResetRelOptions => {
            e.token(TokenKind::RESET_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            if let Some(ref def) = cmd.def {
                emit_node(e, def); // List of option names
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
                emit_identifier_maybe_quoted(e, &cmd.name);
            }
            e.space();
            e.token(TokenKind::SET_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            if let Some(ref def) = cmd.def {
                emit_node(e, def); // List of DefElem nodes
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
                emit_identifier_maybe_quoted(e, &cmd.name);
            }
            e.space();
            e.token(TokenKind::RESET_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            if let Some(ref def) = cmd.def {
                emit_node(e, def); // List of option names
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
                emit_identifier_maybe_quoted(e, &cmd.name);
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
                emit_node(e, def);
            }
        }
        AlterTableType::AtSetStorage => {
            // ALTER COLUMN col SET STORAGE {PLAIN|EXTERNAL|EXTENDED|MAIN}
            e.token(TokenKind::ALTER_KW);
            e.space();
            e.token(TokenKind::COLUMN_KW);
            if !cmd.name.is_empty() {
                e.space();
                emit_identifier_maybe_quoted(e, &cmd.name);
            }
            e.space();
            e.token(TokenKind::SET_KW);
            e.space();
            e.token(TokenKind::STORAGE_KW);
            if let Some(ref def) = cmd.def {
                e.space();
                emit_node(e, def);
            }
        }
        AlterTableType::AtSetCompression => {
            // ALTER COLUMN col SET COMPRESSION method
            e.token(TokenKind::ALTER_KW);
            e.space();
            e.token(TokenKind::COLUMN_KW);
            if !cmd.name.is_empty() {
                e.space();
                emit_identifier_maybe_quoted(e, &cmd.name);
            }
            e.space();
            e.token(TokenKind::SET_KW);
            e.space();
            e.token(TokenKind::COMPRESSION_KW);
            if let Some(ref def) = cmd.def {
                e.space();
                emit_node(e, def);
            }
        }
        AlterTableType::AtClusterOn => {
            e.token(TokenKind::CLUSTER_KW);
            e.space();
            e.token(TokenKind::ON_KW);
            if !cmd.name.is_empty() {
                e.space();
                emit_identifier_maybe_quoted(e, &cmd.name);
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
                emit_identifier_maybe_quoted(e, &cmd.name);
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
                emit_node(e, def);
            }
        }
        AlterTableType::AtDropInherit => {
            e.token(TokenKind::NO_KW);
            e.space();
            e.token(TokenKind::INHERIT_KW);
            if let Some(ref def) = cmd.def {
                e.space();
                emit_node(e, def);
            }
        }
        AlterTableType::AtAddOf => {
            e.token(TokenKind::OF_KW);
            if let Some(ref def) = cmd.def {
                e.space();
                emit_node(e, def);
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
                emit_node(e, def); // PartitionCmd node
            }
        }
        AlterTableType::AtDetachPartition => {
            e.token(TokenKind::DETACH_KW);
            e.space();
            e.token(TokenKind::PARTITION_KW);
            if let Some(ref def) = cmd.def {
                e.line(LineType::SoftOrSpace);
                emit_node(e, def); // PartitionCmd node
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
                emit_identifier_maybe_quoted(e, &cmd.name);
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
                emit_identifier_maybe_quoted(e, &cmd.name);
            }
        }
        AlterTableType::AtEnableRule => {
            e.token(TokenKind::ENABLE_KW);
            e.space();
            e.token(TokenKind::RULE_KW);
            if !cmd.name.is_empty() {
                e.space();
                emit_identifier_maybe_quoted(e, &cmd.name);
            }
        }
        AlterTableType::AtDisableRule => {
            e.token(TokenKind::DISABLE_KW);
            e.space();
            e.token(TokenKind::RULE_KW);
            if !cmd.name.is_empty() {
                e.space();
                emit_identifier_maybe_quoted(e, &cmd.name);
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
                emit_identifier_maybe_quoted(e, &cmd.name);
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
                emit_identifier_maybe_quoted(e, &cmd.name);
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
                emit_identifier_maybe_quoted(e, &cmd.name);
            }
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::ADD_KW);
            e.space();
            // The def is a Constraint containing the full identity definition
            // Just emit the constraint which handles GENERATED ALWAYS/BY DEFAULT AS IDENTITY
            if let Some(ref def) = cmd.def {
                emit_node(e, def);
            }
        }
        AlterTableType::AtSetIdentity => {
            // ALTER COLUMN col SET GENERATED BY DEFAULT/ALWAYS or SET seq options
            e.token(TokenKind::ALTER_KW);
            e.space();
            e.token(TokenKind::COLUMN_KW);
            if !cmd.name.is_empty() {
                e.space();
                emit_identifier_maybe_quoted(e, &cmd.name);
            }
            // def is a List of DefElem options. Every option carries its own SET prefix except
            // RESTART, which Postgres rejects behind SET.
            if let Some(ref def) = cmd.def
                && let Some(NodeEnum::List(list)) = &def.node
            {
                e.group_start();
                e.indent_start();
                for opt in &list.items {
                    if let Some(NodeEnum::DefElem(de)) = &opt.node {
                        e.line(LineType::SoftOrSpace);
                        if de.defname != "restart" {
                            e.token(TokenKind::SET_KW);
                            e.space();
                        }
                        // Handle special "generated" option that specifies ALWAYS/BY DEFAULT
                        if de.defname == "generated" {
                            e.token(TokenKind::GENERATED_KW);
                            e.space();
                            // arg is an Integer - 97='a' for ALWAYS, 100='d' for BY DEFAULT
                            if let Some(ref arg) = de.arg
                                && let Some(NodeEnum::Integer(i)) = &arg.node
                            {
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
                        } else {
                            // Emit other sequence options
                            emit_sequence_option(e, de);
                        }
                    }
                }
                e.indent_end();
                e.group_end();
            }
        }
        AlterTableType::AtDropIdentity => {
            // ALTER COLUMN col DROP IDENTITY [IF EXISTS]
            e.token(TokenKind::ALTER_KW);
            e.space();
            e.token(TokenKind::COLUMN_KW);
            if !cmd.name.is_empty() {
                e.space();
                emit_identifier_maybe_quoted(e, &cmd.name);
            }
            e.space();
            e.token(TokenKind::DROP_KW);
            e.space();
            e.token(TokenKind::IDENTITY_KW);
            if cmd.missing_ok {
                e.space();
                emit_if_exists(e);
            }
        }
        AlterTableType::AtAlterConstraint => {
            // ALTER CONSTRAINT constraint_name [DEFERRABLE | NOT DEFERRABLE] [INITIALLY DEFERRED | INITIALLY IMMEDIATE]
            e.token(TokenKind::ALTER_KW);
            e.space();
            e.token(TokenKind::CONSTRAINT_KW);
            e.space();

            // constraint name comes from the def (Constraint node)
            if let Some(ref def) = cmd.def
                && let Some(NodeEnum::Constraint(c)) = &def.node
            {
                emit_identifier_maybe_quoted(e, &c.conname);

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
                emit_identifier_maybe_quoted(e, &cmd.name);
            }
            e.space();
            e.token(TokenKind::OPTIONS_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            if let Some(ref def) = cmd.def
                && let Some(NodeEnum::List(list)) = &def.node
            {
                for (i, opt) in list.items.iter().enumerate() {
                    if i > 0 {
                        e.token(TokenKind::COMMA);
                        e.space();
                    }
                    if let Some(NodeEnum::DefElem(de)) = &opt.node {
                        emit_options_def_elem(e, de);
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
            if let Some(ref def) = cmd.def
                && let Some(NodeEnum::List(list)) = &def.node
            {
                for (i, opt) in list.items.iter().enumerate() {
                    if i > 0 {
                        e.token(TokenKind::COMMA);
                        e.space();
                    }
                    if let Some(NodeEnum::DefElem(de)) = &opt.node {
                        emit_options_def_elem(e, de);
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
