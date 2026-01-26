use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind, LineType};

use super::node_list::emit_dot_separated_list;
use pgls_query::{
    NodeEnum,
    protobuf::{AlterObjectSchemaStmt, ObjectType},
};

pub(super) fn emit_alter_object_schema_stmt(e: &mut EventEmitter, n: &AlterObjectSchemaStmt) {
    e.group_start(GroupKind::AlterObjectSchemaStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();

    // Emit object type
    let object_type = n.object_type();
    match object_type {
        ObjectType::ObjectTable => e.token(TokenKind::TABLE_KW),
        ObjectType::ObjectSequence => e.token(TokenKind::SEQUENCE_KW),
        ObjectType::ObjectView => e.token(TokenKind::VIEW_KW),
        ObjectType::ObjectMatview => {
            e.token(TokenKind::MATERIALIZED_KW);
            e.space();
            e.token(TokenKind::VIEW_KW);
        }
        ObjectType::ObjectIndex => e.token(TokenKind::INDEX_KW),
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
        ObjectType::ObjectForeignTable => {
            e.token(TokenKind::FOREIGN_KW);
            e.space();
            e.token(TokenKind::TABLE_KW);
        }
        ObjectType::ObjectCollation => e.token(TokenKind::COLLATION_KW),
        ObjectType::ObjectConversion => e.token(TokenKind::CONVERSION_KW),
        ObjectType::ObjectStatisticExt => e.token(TokenKind::STATISTICS_KW),
        ObjectType::ObjectTsconfiguration => {
            e.token(TokenKind::TEXT_KW);
            e.space();
            e.token(TokenKind::SEARCH_KW);
            e.space();
            e.token(TokenKind::CONFIGURATION_KW);
        }
        ObjectType::ObjectTsdictionary => {
            e.token(TokenKind::TEXT_KW);
            e.space();
            e.token(TokenKind::SEARCH_KW);
            e.space();
            e.token(TokenKind::DICTIONARY_KW);
        }
        ObjectType::ObjectTsparser => {
            e.token(TokenKind::TEXT_KW);
            e.space();
            e.token(TokenKind::SEARCH_KW);
            e.space();
            e.token(TokenKind::PARSER_KW);
        }
        ObjectType::ObjectTstemplate => {
            e.token(TokenKind::TEXT_KW);
            e.space();
            e.token(TokenKind::SEARCH_KW);
            e.space();
            e.token(TokenKind::TEMPLATE_KW);
        }
        ObjectType::ObjectFunction => e.token(TokenKind::FUNCTION_KW),
        ObjectType::ObjectProcedure => e.token(TokenKind::PROCEDURE_KW),
        ObjectType::ObjectRoutine => e.token(TokenKind::ROUTINE_KW),
        ObjectType::ObjectAggregate => e.token(TokenKind::AGGREGATE_KW),
        ObjectType::ObjectOperator => e.token(TokenKind::OPERATOR_KW),
        ObjectType::ObjectType => e.token(TokenKind::TYPE_KW),
        ObjectType::ObjectDomain => e.token(TokenKind::DOMAIN_KW),
        ObjectType::ObjectExtension => e.token(TokenKind::EXTENSION_KW),
        _ => {
            debug_assert!(
                false,
                "Unhandled ObjectType in AlterObjectSchemaStmt: {object_type:?}"
            );
            e.token(TokenKind::IDENT("UNKNOWN".to_string()));
        }
    }
    e.space();

    if n.missing_ok {
        e.token(TokenKind::IF_KW);
        e.space();
        e.token(TokenKind::EXISTS_KW);
        e.space();
    }

    // Emit object name
    if let Some(ref relation) = n.relation {
        super::emit_range_var(e, relation);
    } else if let Some(ref object) = n.object {
        match n.object_type() {
            ObjectType::ObjectOpclass | ObjectType::ObjectOpfamily => {
                emit_operator_collection_object(e, object)
            }
            ObjectType::ObjectAggregate => {
                // Aggregate needs (*) for "any argument types"
                if let Some(NodeEnum::ObjectWithArgs(owa)) = object.node.as_ref() {
                    super::emit_object_with_args_for_aggregate(e, owa);
                } else {
                    super::emit_node(object, e);
                }
            }
            _ => super::emit_node(object, e),
        }
    }

    // Emit new schema
    if !n.newschema.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::SET_KW);
        e.space();
        e.token(TokenKind::SCHEMA_KW);
        e.space();
        e.token(TokenKind::IDENT(n.newschema.clone()));
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}

fn emit_operator_collection_object(e: &mut EventEmitter, object: &pgls_query::Node) {
    if let Some(pgls_query::NodeEnum::List(list)) = &object.node
        && list.items.len() >= 2
    {
        let (method_node, name_nodes) = list.items.split_first().unwrap();
        emit_dot_separated_list(e, name_nodes);
        e.space();
        e.token(TokenKind::USING_KW);
        e.space();
        super::emit_node(method_node, e);
        return;
    }

    super::emit_node(object, e);
}
