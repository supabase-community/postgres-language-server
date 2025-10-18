use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgt_query::protobuf::{ObjectType, SecLabelStmt};

use super::string::{emit_identifier_maybe_quoted, emit_keyword, emit_single_quoted_str};

pub(super) fn emit_sec_label_stmt(e: &mut EventEmitter, n: &SecLabelStmt) {
    e.group_start(GroupKind::SecLabelStmt);

    emit_keyword(e, "SECURITY");
    e.space();
    emit_keyword(e, "LABEL");

    // Emit FOR provider if present
    if !n.provider.is_empty() {
        e.space();
        e.token(TokenKind::FOR_KW);
        e.space();
        emit_identifier_maybe_quoted(e, &n.provider);
    }

    // Emit ON object_type object
    e.space();
    e.token(TokenKind::ON_KW);
    e.space();

    // Map object type to SQL keyword
    let objtype_tokens: &[&str] = match ObjectType::try_from(n.objtype) {
        Ok(ObjectType::ObjectTable) => &["TABLE"],
        Ok(ObjectType::ObjectSequence) => &["SEQUENCE"],
        Ok(ObjectType::ObjectView) => &["VIEW"],
        Ok(ObjectType::ObjectColumn) => &["COLUMN"],
        Ok(ObjectType::ObjectDatabase) => &["DATABASE"],
        Ok(ObjectType::ObjectSchema) => &["SCHEMA"],
        Ok(ObjectType::ObjectFunction) => &["FUNCTION"],
        Ok(ObjectType::ObjectProcedure) => &["PROCEDURE"],
        Ok(ObjectType::ObjectRoutine) => &["ROUTINE"],
        Ok(ObjectType::ObjectType) => &["TYPE"],
        Ok(ObjectType::ObjectDomain) => &["DOMAIN"],
        Ok(ObjectType::ObjectAggregate) => &["AGGREGATE"],
        Ok(ObjectType::ObjectRole) => &["ROLE"],
        Ok(ObjectType::ObjectTablespace) => &["TABLESPACE"],
        Ok(ObjectType::ObjectFdw) => &["FOREIGN", "DATA", "WRAPPER"],
        Ok(ObjectType::ObjectForeignServer) => &["FOREIGN", "SERVER"],
        Ok(ObjectType::ObjectLanguage) => &["LANGUAGE"],
        Ok(ObjectType::ObjectLargeobject) => &["LARGE", "OBJECT"],
        _ => &["TABLE"],
    };

    for (idx, token) in objtype_tokens.iter().enumerate() {
        if idx > 0 {
            e.space();
        }
        emit_keyword(e, token);
    }
    e.space();

    // Emit object name
    if let Some(ref object) = n.object {
        super::emit_node(object, e);
    }

    // Emit IS 'label'
    e.space();
    e.token(TokenKind::IS_KW);
    e.space();
    emit_single_quoted_str(e, &n.label);

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
