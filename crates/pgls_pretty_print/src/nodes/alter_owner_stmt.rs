use pgls_query::{
    NodeEnum,
    protobuf::{AlterOwnerStmt, ObjectType},
};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

use super::node_list::emit_dot_separated_list;

pub(super) fn emit_alter_owner_stmt(e: &mut EventEmitter, n: &AlterOwnerStmt) {
    e.group_start(GroupKind::AlterOwnerStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();

    let object_type = ObjectType::try_from(n.object_type).unwrap_or(ObjectType::Undefined);
    emit_object_type(e, object_type);

    match object_type {
        ObjectType::ObjectOpfamily | ObjectType::ObjectOpclass => {
            if let Some(ref object) = n.object {
                e.space();
                emit_owner_operator_collection(e, object);
            }
        }
        ObjectType::ObjectAggregate => {
            // Aggregate needs (*) for "any argument types"
            if let Some(ref object) = n.object {
                e.space();
                if let Some(NodeEnum::ObjectWithArgs(owa)) = object.node.as_ref() {
                    super::emit_object_with_args_for_aggregate(e, owa);
                } else {
                    emit_owner_object(e, object);
                }
            }
        }
        _ => {
            if let Some(ref relation) = n.relation {
                e.space();
                super::emit_range_var(e, relation);
            } else if let Some(ref object) = n.object {
                e.space();
                emit_owner_object(e, object);
            }
        }
    }

    e.line(crate::emitter::LineType::SoftOrSpace);
    e.token(TokenKind::OWNER_KW);
    e.space();
    e.token(TokenKind::TO_KW);

    if let Some(ref newowner) = n.newowner {
        e.space();
        super::emit_role_spec(e, newowner);
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}

fn emit_object_type(e: &mut EventEmitter, object_type: ObjectType) {
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
        _ => e.token(TokenKind::TABLE_KW),
    }
}

fn emit_owner_object(e: &mut EventEmitter, object: &pgls_query::Node) {
    match &object.node {
        Some(NodeEnum::List(list)) => emit_dot_separated_list(e, &list.items),
        _ => super::emit_node(object, e),
    }
}

fn emit_owner_operator_collection(e: &mut EventEmitter, object: &pgls_query::Node) {
    if let Some(NodeEnum::List(list)) = &object.node
        && list.items.len() >= 2
    {
        let (method_node, name_nodes) = list.items.split_first().unwrap();
        if !name_nodes.is_empty() {
            emit_dot_separated_list(e, name_nodes);
            e.space();
            e.token(TokenKind::USING_KW);
            e.space();
            super::emit_node(method_node, e);
            return;
        }
    }

    emit_owner_object(e, object);
}
