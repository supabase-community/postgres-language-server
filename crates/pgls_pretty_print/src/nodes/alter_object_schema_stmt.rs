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
    let object_type_str = match object_type {
        ObjectType::ObjectTable => "TABLE",
        ObjectType::ObjectSequence => "SEQUENCE",
        ObjectType::ObjectView => "VIEW",
        ObjectType::ObjectMatview => "MATERIALIZED VIEW",
        ObjectType::ObjectIndex => "INDEX",
        ObjectType::ObjectOpclass => "OPERATOR CLASS",
        ObjectType::ObjectOpfamily => "OPERATOR FAMILY",
        ObjectType::ObjectForeignTable => "FOREIGN TABLE",
        ObjectType::ObjectCollation => "COLLATION",
        ObjectType::ObjectConversion => "CONVERSION",
        ObjectType::ObjectStatisticExt => "STATISTICS",
        ObjectType::ObjectTsconfiguration => "TEXT SEARCH CONFIGURATION",
        ObjectType::ObjectTsdictionary => "TEXT SEARCH DICTIONARY",
        ObjectType::ObjectTsparser => "TEXT SEARCH PARSER",
        ObjectType::ObjectTstemplate => "TEXT SEARCH TEMPLATE",
        ObjectType::ObjectFunction => "FUNCTION",
        ObjectType::ObjectProcedure => "PROCEDURE",
        ObjectType::ObjectRoutine => "ROUTINE",
        ObjectType::ObjectAggregate => "AGGREGATE",
        ObjectType::ObjectOperator => "OPERATOR",
        ObjectType::ObjectType => "TYPE",
        ObjectType::ObjectDomain => "DOMAIN",
        ObjectType::ObjectExtension => "EXTENSION",
        _ => {
            debug_assert!(
                false,
                "Unhandled ObjectType in AlterObjectSchemaStmt: {object_type:?}"
            );
            "UNKNOWN"
        }
    };

    e.token(TokenKind::IDENT(object_type_str.to_string()));
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
        e.token(TokenKind::IDENT("SCHEMA".to_string()));
        e.space();
        e.token(TokenKind::IDENT(n.newschema.clone()));
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}

fn emit_operator_collection_object(e: &mut EventEmitter, object: &pgls_query::Node) {
    if let Some(pgls_query::NodeEnum::List(list)) = &object.node {
        if list.items.len() >= 2 {
            let (method_node, name_nodes) = list.items.split_first().unwrap();
            emit_dot_separated_list(e, name_nodes);
            e.space();
            e.token(TokenKind::USING_KW);
            e.space();
            super::emit_node(method_node, e);
            return;
        }
    }

    super::emit_node(object, e);
}
