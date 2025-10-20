use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};
use pgt_query::protobuf::{AlterObjectSchemaStmt, ObjectType};

pub(super) fn emit_alter_object_schema_stmt(e: &mut EventEmitter, n: &AlterObjectSchemaStmt) {
    e.group_start(GroupKind::AlterObjectSchemaStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();

    // Emit object type
    let object_type_str = match n.object_type() {
        ObjectType::ObjectTable => "TABLE",
        ObjectType::ObjectSequence => "SEQUENCE",
        ObjectType::ObjectView => "VIEW",
        ObjectType::ObjectMatview => "MATERIALIZED VIEW",
        ObjectType::ObjectIndex => "INDEX",
        ObjectType::ObjectForeignTable => "FOREIGN TABLE",
        ObjectType::ObjectCollation => "COLLATION",
        ObjectType::ObjectConversion => "CONVERSION",
        ObjectType::ObjectStatisticExt => "STATISTICS",
        ObjectType::ObjectTsconfiguration => "TEXT SEARCH CONFIGURATION",
        ObjectType::ObjectTsdictionary => "TEXT SEARCH DICTIONARY",
        ObjectType::ObjectFunction => "FUNCTION",
        ObjectType::ObjectProcedure => "PROCEDURE",
        ObjectType::ObjectRoutine => "ROUTINE",
        ObjectType::ObjectAggregate => "AGGREGATE",
        ObjectType::ObjectOperator => "OPERATOR",
        ObjectType::ObjectType => "TYPE",
        ObjectType::ObjectDomain => "DOMAIN",
        _ => "UNKNOWN",
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
        super::emit_node(object, e);
    }

    // Emit new schema
    if !n.newschema.is_empty() {
        e.space();
        e.token(TokenKind::SET_KW);
        e.space();
        e.token(TokenKind::IDENT("SCHEMA".to_string()));
        e.space();
        e.token(TokenKind::IDENT(n.newschema.clone()));
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
