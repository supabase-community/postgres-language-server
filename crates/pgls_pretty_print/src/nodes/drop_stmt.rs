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
    emit_object_type_keywords(e, object_type);

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
            super::string::emit_identifier_maybe_quoted(e, &s.sval);
        } else {
            super::emit_node(item, e);
        }
    });
}

fn emit_drop_cast_object(node: &pgls_query::protobuf::Node, e: &mut EventEmitter) {
    if let Some(pgls_query::NodeEnum::List(list)) = node.node.as_ref()
        && list.items.len() == 2
    {
        e.token(TokenKind::L_PAREN);
        super::emit_node(&list.items[0], e);
        e.space();
        e.token(TokenKind::AS_KW);
        e.space();
        super::emit_node(&list.items[1], e);
        e.token(TokenKind::R_PAREN);
        return;
    }

    // Fallback for unexpected structure
    super::emit_node(node, e);
}

/// Emit DROP OPERATOR CLASS/FAMILY object.
/// Format: name USING access_method
/// The list is [access_method, name_parts...]
fn emit_drop_opclass_object(node: &pgls_query::protobuf::Node, e: &mut EventEmitter) {
    if let Some(pgls_query::NodeEnum::List(list)) = node.node.as_ref()
        && list.items.len() >= 2
    {
        // First element is the access method name
        // Remaining elements are the operator class/family name parts
        let name_parts = &list.items[1..];
        emit_dot_separated_identifiers(e, name_parts);
        e.space();
        e.token(TokenKind::USING_KW);
        e.space();
        if let Some(pgls_query::NodeEnum::String(s)) = list.items[0].node.as_ref() {
            super::string::emit_identifier_maybe_quoted(e, &s.sval);
        }
        return;
    }

    // Fallback for unexpected structure
    super::emit_node(node, e);
}

/// Emit DROP RULE/TRIGGER/POLICY object.
/// Format: name ON table_name
/// The list is [table_name_parts..., object_name] - last item is the object name
fn emit_drop_on_object(node: &pgls_query::protobuf::Node, e: &mut EventEmitter) {
    if let Some(pgls_query::NodeEnum::List(list)) = node.node.as_ref()
        && list.items.len() >= 2
    {
        // Last element is the object name (trigger/rule/policy)
        // All previous elements are the table name parts (schema, table, etc.)
        let (table_parts, object_name_item) = list.items.split_at(list.items.len() - 1);
        let object_name = &object_name_item[0];

        // Emit object name first
        if let Some(pgls_query::NodeEnum::String(s)) = object_name.node.as_ref() {
            super::string::emit_identifier_maybe_quoted(e, &s.sval);
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

    // Fallback for unexpected structure
    super::emit_node(node, e);
}

/// Emit the object type keywords for DROP statements
fn emit_object_type_keywords(e: &mut EventEmitter, object_type: ObjectType) {
    match object_type {
        ObjectType::ObjectTable => e.token(TokenKind::TABLE_KW),
        ObjectType::ObjectSequence => e.token(TokenKind::SEQUENCE_KW),
        ObjectType::ObjectView => e.token(TokenKind::VIEW_KW),
        ObjectType::ObjectMatview => {
            e.token(TokenKind::MATERIALIZED_KW);
            e.space();
            e.token(TokenKind::VIEW_KW);
        }
        ObjectType::ObjectForeignTable => {
            e.token(TokenKind::FOREIGN_KW);
            e.space();
            e.token(TokenKind::TABLE_KW);
        }
        ObjectType::ObjectIndex => e.token(TokenKind::INDEX_KW),
        ObjectType::ObjectDatabase => e.token(TokenKind::DATABASE_KW),
        ObjectType::ObjectSchema => e.token(TokenKind::SCHEMA_KW),
        ObjectType::ObjectTablespace => e.token(TokenKind::TABLESPACE_KW),
        ObjectType::ObjectFunction => e.token(TokenKind::FUNCTION_KW),
        ObjectType::ObjectProcedure => e.token(TokenKind::PROCEDURE_KW),
        ObjectType::ObjectRoutine => e.token(TokenKind::ROUTINE_KW),
        ObjectType::ObjectType => e.token(TokenKind::TYPE_KW),
        ObjectType::ObjectOperator => e.token(TokenKind::OPERATOR_KW),
        ObjectType::ObjectAggregate => e.token(TokenKind::AGGREGATE_KW),
        ObjectType::ObjectOpclass => {
            e.token(TokenKind::OPERATOR_KW);
            e.space();
            e.token(TokenKind::CLASS_KW);
        }
        ObjectType::ObjectOpfamily => {
            e.token(TokenKind::OPERATOR_KW);
            e.space();
            e.token(TokenKind::FAMILY_KW);
        }
        ObjectType::ObjectConversion => e.token(TokenKind::CONVERSION_KW),
        ObjectType::ObjectCollation => e.token(TokenKind::COLLATION_KW),
        ObjectType::ObjectDomain => e.token(TokenKind::DOMAIN_KW),
        ObjectType::ObjectExtension => e.token(TokenKind::EXTENSION_KW),
        ObjectType::ObjectLanguage => e.token(TokenKind::LANGUAGE_KW),
        ObjectType::ObjectPublication => e.token(TokenKind::PUBLICATION_KW),
        ObjectType::ObjectSubscription => e.token(TokenKind::SUBSCRIPTION_KW),
        ObjectType::ObjectCast => e.token(TokenKind::CAST_KW),
        ObjectType::ObjectFdw => {
            e.token(TokenKind::FOREIGN_KW);
            e.space();
            e.token(TokenKind::DATA_KW);
            e.space();
            e.token(TokenKind::WRAPPER_KW);
        }
        ObjectType::ObjectForeignServer => e.token(TokenKind::SERVER_KW),
        ObjectType::ObjectAccessMethod => {
            e.token(TokenKind::ACCESS_KW);
            e.space();
            e.token(TokenKind::METHOD_KW);
        }
        ObjectType::ObjectLargeobject => {
            e.token(TokenKind::LARGE_KW);
            e.space();
            e.token(TokenKind::OBJECT_KW);
        }
        ObjectType::ObjectTsparser => {
            e.token(TokenKind::TEXT_KW);
            e.space();
            e.token(TokenKind::SEARCH_KW);
            e.space();
            e.token(TokenKind::PARSER_KW);
        }
        ObjectType::ObjectTsdictionary => {
            e.token(TokenKind::TEXT_KW);
            e.space();
            e.token(TokenKind::SEARCH_KW);
            e.space();
            e.token(TokenKind::DICTIONARY_KW);
        }
        ObjectType::ObjectTstemplate => {
            e.token(TokenKind::TEXT_KW);
            e.space();
            e.token(TokenKind::SEARCH_KW);
            e.space();
            e.token(TokenKind::TEMPLATE_KW);
        }
        ObjectType::ObjectTsconfiguration => {
            e.token(TokenKind::TEXT_KW);
            e.space();
            e.token(TokenKind::SEARCH_KW);
            e.space();
            e.token(TokenKind::CONFIGURATION_KW);
        }
        ObjectType::ObjectStatisticExt => e.token(TokenKind::STATISTICS_KW),
        ObjectType::ObjectPolicy => e.token(TokenKind::POLICY_KW),
        ObjectType::ObjectRule => e.token(TokenKind::RULE_KW),
        ObjectType::ObjectTrigger => e.token(TokenKind::TRIGGER_KW),
        ObjectType::ObjectEventTrigger => {
            e.token(TokenKind::EVENT_KW);
            e.space();
            e.token(TokenKind::TRIGGER_KW);
        }
        ObjectType::ObjectUserMapping => {
            e.token(TokenKind::USER_KW);
            e.space();
            e.token(TokenKind::MAPPING_KW);
        }
        ObjectType::ObjectTransform => e.token(TokenKind::TRANSFORM_KW),
        // Fallback for any unhandled object types
        _ => e.token(TokenKind::TABLE_KW),
    }
}
