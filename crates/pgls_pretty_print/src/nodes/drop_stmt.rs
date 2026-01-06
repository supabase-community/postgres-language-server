use pgls_query::protobuf::{DropBehavior, DropStmt, ObjectType};

use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind, LineType};

use super::node_list::{emit_comma_separated_list, emit_dot_separated_list_with};

pub(super) fn emit_drop_stmt(e: &mut EventEmitter, n: &DropStmt) {
    e.group_start(GroupKind::DropStmt);

    e.token(TokenKind::DROP_KW);
    e.space();

    // Object type - use typed enum accessor
    let object_type = n.remove_type();
    let object_type_str = match object_type {
        ObjectType::ObjectTable => "TABLE",
        ObjectType::ObjectIndex => "INDEX",
        ObjectType::ObjectSequence => "SEQUENCE",
        ObjectType::ObjectView => "VIEW",
        ObjectType::ObjectSchema => "SCHEMA",
        ObjectType::ObjectFunction => "FUNCTION",
        ObjectType::ObjectProcedure => "PROCEDURE",
        ObjectType::ObjectRoutine => "ROUTINE",
        ObjectType::ObjectAggregate => "AGGREGATE",
        ObjectType::ObjectOperator => "OPERATOR",
        ObjectType::ObjectOpclass => "OPERATOR CLASS",
        ObjectType::ObjectOpfamily => "OPERATOR FAMILY",
        ObjectType::ObjectType => "TYPE",
        ObjectType::ObjectDomain => "DOMAIN",
        ObjectType::ObjectCollation => "COLLATION",
        ObjectType::ObjectConversion => "CONVERSION",
        ObjectType::ObjectTrigger => "TRIGGER",
        ObjectType::ObjectRule => "RULE",
        ObjectType::ObjectExtension => "EXTENSION",
        ObjectType::ObjectForeignTable => "FOREIGN TABLE",
        ObjectType::ObjectMatview => "MATERIALIZED VIEW",
        ObjectType::ObjectRole => "ROLE",
        ObjectType::ObjectDatabase => "DATABASE",
        ObjectType::ObjectTablespace => "TABLESPACE",
        ObjectType::ObjectFdw => "FOREIGN DATA WRAPPER",
        ObjectType::ObjectForeignServer => "SERVER",
        ObjectType::ObjectUserMapping => "USER MAPPING",
        ObjectType::ObjectAccessMethod => "ACCESS METHOD",
        ObjectType::ObjectPublication => "PUBLICATION",
        ObjectType::ObjectSubscription => "SUBSCRIPTION",
        ObjectType::ObjectPolicy => "POLICY",
        ObjectType::ObjectEventTrigger => "EVENT TRIGGER",
        ObjectType::ObjectTransform => "TRANSFORM",
        ObjectType::ObjectCast => "CAST",
        ObjectType::ObjectStatisticExt => "STATISTICS",
        ObjectType::ObjectLanguage => "LANGUAGE",
        ObjectType::ObjectTsparser => "TEXT SEARCH PARSER",
        ObjectType::ObjectTsdictionary => "TEXT SEARCH DICTIONARY",
        ObjectType::ObjectTstemplate => "TEXT SEARCH TEMPLATE",
        ObjectType::ObjectTsconfiguration => "TEXT SEARCH CONFIGURATION",
        _ => {
            debug_assert!(false, "Unhandled ObjectType in DropStmt: {object_type:?}");
            "UNKNOWN"
        }
    };

    e.token(TokenKind::IDENT(object_type_str.to_string()));

    // CONCURRENTLY for indexes
    if n.concurrent && object_type == ObjectType::ObjectIndex {
        e.space();
        e.token(TokenKind::CONCURRENTLY_KW);
    }

    // IF EXISTS
    if n.missing_ok {
        e.space();
        e.token(TokenKind::IF_KW);
        e.space();
        e.token(TokenKind::EXISTS_KW);
    }

    // Object names - indent list when it breaks to multiple lines
    if !n.objects.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.indent_start();
        match object_type {
            ObjectType::ObjectCast => {
                emit_comma_separated_list(e, &n.objects, emit_drop_cast_object);
            }
            ObjectType::ObjectOpclass | ObjectType::ObjectOpfamily => {
                // DROP OPERATOR CLASS/FAMILY uses: name USING access_method
                emit_comma_separated_list(e, &n.objects, emit_drop_opclass_object);
            }
            ObjectType::ObjectRule | ObjectType::ObjectTrigger | ObjectType::ObjectPolicy => {
                // DROP RULE/TRIGGER/POLICY uses: name ON table_name
                emit_comma_separated_list(e, &n.objects, emit_drop_on_object);
            }
            ObjectType::ObjectAggregate => {
                // DROP AGGREGATE needs (*) for "any argument types"
                emit_comma_separated_list(e, &n.objects, |node, e| {
                    if let Some(pgls_query::NodeEnum::ObjectWithArgs(owa)) = node.node.as_ref() {
                        super::emit_object_with_args_for_aggregate(e, owa);
                    } else {
                        super::emit_node(node, e);
                    }
                });
            }
            _ => {
                emit_comma_separated_list(e, &n.objects, |node, e| {
                    if let Some(pgls_query::NodeEnum::List(list)) = node.node.as_ref() {
                        emit_dot_separated_identifiers(e, &list.items);
                    } else {
                        super::emit_node(node, e);
                    }
                });
            }
        }
        e.indent_end();
    }

    // CASCADE/RESTRICT
    if n.behavior() == DropBehavior::DropCascade {
        e.space();
        e.token(TokenKind::CASCADE_KW);
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}

fn emit_dot_separated_identifiers(e: &mut EventEmitter, items: &[pgls_query::protobuf::Node]) {
    emit_dot_separated_list_with(e, items, |item, e| {
        if let Some(pgls_query::NodeEnum::String(s)) = item.node.as_ref() {
            super::string::emit_identifier(e, &s.sval);
        } else {
            super::emit_node(item, e);
        }
    });
}

fn emit_drop_cast_object(node: &pgls_query::protobuf::Node, e: &mut EventEmitter) {
    if let Some(pgls_query::NodeEnum::List(list)) = node.node.as_ref() {
        if list.items.len() == 2 {
            e.token(TokenKind::L_PAREN);
            super::emit_node(&list.items[0], e);
            e.space();
            e.token(TokenKind::AS_KW);
            e.space();
            super::emit_node(&list.items[1], e);
            e.token(TokenKind::R_PAREN);
            return;
        }
    }

    // Fallback for unexpected structure
    super::emit_node(node, e);
}

/// Emit DROP OPERATOR CLASS/FAMILY object.
/// Format: name USING access_method
/// The list is [access_method, name_parts...]
fn emit_drop_opclass_object(node: &pgls_query::protobuf::Node, e: &mut EventEmitter) {
    if let Some(pgls_query::NodeEnum::List(list)) = node.node.as_ref() {
        if list.items.len() >= 2 {
            // First element is the access method name
            // Remaining elements are the operator class/family name parts
            let name_parts = &list.items[1..];
            emit_dot_separated_identifiers(e, name_parts);
            e.space();
            e.token(TokenKind::IDENT("USING".to_string()));
            e.space();
            if let Some(pgls_query::NodeEnum::String(s)) = list.items[0].node.as_ref() {
                super::string::emit_identifier(e, &s.sval);
            }
            return;
        }
    }

    // Fallback for unexpected structure
    super::emit_node(node, e);
}

/// Emit DROP RULE/TRIGGER/POLICY object.
/// Format: name ON table_name
/// The list is [table_name_parts..., object_name] - last item is the object name
fn emit_drop_on_object(node: &pgls_query::protobuf::Node, e: &mut EventEmitter) {
    if let Some(pgls_query::NodeEnum::List(list)) = node.node.as_ref() {
        if list.items.len() >= 2 {
            // Last element is the object name (trigger/rule/policy)
            // All previous elements are the table name parts (schema, table, etc.)
            let (table_parts, object_name_item) = list.items.split_at(list.items.len() - 1);
            let object_name = &object_name_item[0];

            // Emit object name first
            if let Some(pgls_query::NodeEnum::String(s)) = object_name.node.as_ref() {
                super::string::emit_identifier(e, &s.sval);
            } else {
                super::emit_node(object_name, e);
            }

            e.space();
            e.token(TokenKind::ON_KW);
            e.space();

            // Emit table name parts dot-separated
            emit_dot_separated_identifiers(e, table_parts);
            return;
        }
    }

    // Fallback for unexpected structure
    super::emit_node(node, e);
}
