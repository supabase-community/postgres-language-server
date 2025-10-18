use pgt_query::protobuf::{DropBehavior, DropStmt, ObjectType};

use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};

use super::node_list::emit_comma_separated_list;

pub(super) fn emit_drop_stmt(e: &mut EventEmitter, n: &DropStmt) {
    e.group_start(GroupKind::DropStmt);

    e.token(TokenKind::DROP_KW);
    e.space();

    // Object type
    let object_type_str = match n.remove_type {
        x if x == ObjectType::ObjectTable as i32 => "TABLE",
        x if x == ObjectType::ObjectIndex as i32 => "INDEX",
        x if x == ObjectType::ObjectSequence as i32 => "SEQUENCE",
        x if x == ObjectType::ObjectView as i32 => "VIEW",
        x if x == ObjectType::ObjectSchema as i32 => "SCHEMA",
        x if x == ObjectType::ObjectFunction as i32 => "FUNCTION",
        x if x == ObjectType::ObjectProcedure as i32 => "PROCEDURE",
        x if x == ObjectType::ObjectRoutine as i32 => "ROUTINE",
        x if x == ObjectType::ObjectAggregate as i32 => "AGGREGATE",
        x if x == ObjectType::ObjectOperator as i32 => "OPERATOR",
        x if x == ObjectType::ObjectType as i32 => "TYPE",
        x if x == ObjectType::ObjectDomain as i32 => "DOMAIN",
        x if x == ObjectType::ObjectCollation as i32 => "COLLATION",
        x if x == ObjectType::ObjectConversion as i32 => "CONVERSION",
        x if x == ObjectType::ObjectTrigger as i32 => "TRIGGER",
        x if x == ObjectType::ObjectRule as i32 => "RULE",
        x if x == ObjectType::ObjectExtension as i32 => "EXTENSION",
        x if x == ObjectType::ObjectForeignTable as i32 => "FOREIGN TABLE",
        x if x == ObjectType::ObjectMatview as i32 => "MATERIALIZED VIEW",
        x if x == ObjectType::ObjectRole as i32 => "ROLE",
        x if x == ObjectType::ObjectDatabase as i32 => "DATABASE",
        x if x == ObjectType::ObjectTablespace as i32 => "TABLESPACE",
        x if x == ObjectType::ObjectFdw as i32 => "FOREIGN DATA WRAPPER",
        x if x == ObjectType::ObjectForeignServer as i32 => "SERVER",
        x if x == ObjectType::ObjectUserMapping as i32 => "USER MAPPING",
        x if x == ObjectType::ObjectAccessMethod as i32 => "ACCESS METHOD",
        x if x == ObjectType::ObjectPublication as i32 => "PUBLICATION",
        x if x == ObjectType::ObjectSubscription as i32 => "SUBSCRIPTION",
        x if x == ObjectType::ObjectPolicy as i32 => "POLICY",
        x if x == ObjectType::ObjectEventTrigger as i32 => "EVENT TRIGGER",
        x if x == ObjectType::ObjectTransform as i32 => "TRANSFORM",
        x if x == ObjectType::ObjectCast as i32 => "CAST",
        _ => "UNKNOWN",
    };

    e.token(TokenKind::IDENT(object_type_str.to_string()));

    // CONCURRENTLY for indexes
    if n.concurrent && n.remove_type == ObjectType::ObjectIndex as i32 {
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

    // Object names
    if !n.objects.is_empty() {
        e.space();
        emit_comma_separated_list(e, &n.objects, |node, e| {
            // Objects can be:
            // - List (qualified names like schema.table)
            // - String (simple names)
            // - ObjectWithArgs (for functions/operators)
            // - TypeName (for types)

            if let Some(pgt_query::NodeEnum::List(list)) = node.node.as_ref() {
                // Qualified name: emit as schema.table
                emit_dot_separated_identifiers(e, &list.items);
            } else {
                super::emit_node(node, e);
            }
        });
    }

    // CASCADE/RESTRICT
    if n.behavior == DropBehavior::DropCascade as i32 {
        e.space();
        e.token(TokenKind::CASCADE_KW);
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}

fn emit_dot_separated_identifiers(e: &mut EventEmitter, items: &[pgt_query::protobuf::Node]) {
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            e.token(TokenKind::DOT);
        }

        if let Some(pgt_query::NodeEnum::String(s)) = item.node.as_ref() {
            super::string::emit_identifier(e, &s.sval);
        } else {
            super::emit_node(item, e);
        }
    }
}
