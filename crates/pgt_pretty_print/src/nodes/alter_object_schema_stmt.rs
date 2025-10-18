use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};
use pgt_query::protobuf::{AlterObjectSchemaStmt, ObjectType};

pub(super) fn emit_alter_object_schema_stmt(e: &mut EventEmitter, n: &AlterObjectSchemaStmt) {
    e.group_start(GroupKind::AlterObjectSchemaStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();

    // Emit object type
    let object_type_str = match ObjectType::try_from(n.object_type) {
        Ok(ObjectType::ObjectTable) => "TABLE",
        Ok(ObjectType::ObjectSequence) => "SEQUENCE",
        Ok(ObjectType::ObjectView) => "VIEW",
        Ok(ObjectType::ObjectMatview) => "MATERIALIZED VIEW",
        Ok(ObjectType::ObjectIndex) => "INDEX",
        Ok(ObjectType::ObjectForeignTable) => "FOREIGN TABLE",
        Ok(ObjectType::ObjectCollation) => "COLLATION",
        Ok(ObjectType::ObjectConversion) => "CONVERSION",
        Ok(ObjectType::ObjectStatisticExt) => "STATISTICS",
        Ok(ObjectType::ObjectTsconfiguration) => "TEXT SEARCH CONFIGURATION",
        Ok(ObjectType::ObjectTsdictionary) => "TEXT SEARCH DICTIONARY",
        Ok(ObjectType::ObjectFunction) => "FUNCTION",
        Ok(ObjectType::ObjectProcedure) => "PROCEDURE",
        Ok(ObjectType::ObjectRoutine) => "ROUTINE",
        Ok(ObjectType::ObjectAggregate) => "AGGREGATE",
        Ok(ObjectType::ObjectOperator) => "OPERATOR",
        Ok(ObjectType::ObjectType) => "TYPE",
        Ok(ObjectType::ObjectDomain) => "DOMAIN",
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
