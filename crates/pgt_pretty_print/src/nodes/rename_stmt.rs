use pgt_query::protobuf::{ObjectType, RenameStmt};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_rename_stmt(e: &mut EventEmitter, n: &RenameStmt) {
    e.group_start(GroupKind::RenameStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();

    // ObjectType - map rename_type to SQL keyword using ObjectType enum
    match n.rename_type {
        x if x == ObjectType::ObjectTable as i32 => e.token(TokenKind::TABLE_KW),
        x if x == ObjectType::ObjectSequence as i32 => e.token(TokenKind::SEQUENCE_KW),
        x if x == ObjectType::ObjectView as i32 => e.token(TokenKind::VIEW_KW),
        x if x == ObjectType::ObjectIndex as i32 => e.token(TokenKind::INDEX_KW),
        x if x == ObjectType::ObjectType as i32 => e.token(TokenKind::TYPE_KW),
        x if x == ObjectType::ObjectDomain as i32 => e.token(TokenKind::DOMAIN_KW),
        x if x == ObjectType::ObjectDatabase as i32 => e.token(TokenKind::DATABASE_KW),
        x if x == ObjectType::ObjectSchema as i32 => e.token(TokenKind::SCHEMA_KW),
        x if x == ObjectType::ObjectFunction as i32 => e.token(TokenKind::FUNCTION_KW),
        x if x == ObjectType::ObjectProcedure as i32 => e.token(TokenKind::PROCEDURE_KW),
        x if x == ObjectType::ObjectColumn as i32 => e.token(TokenKind::COLUMN_KW),
        x if x == ObjectType::ObjectMatview as i32 => {
            e.token(TokenKind::MATERIALIZED_KW);
            e.space();
            e.token(TokenKind::VIEW_KW);
        }
        _ => e.token(TokenKind::TABLE_KW), // default fallback
    }

    if n.missing_ok {
        e.space();
        e.token(TokenKind::IF_KW);
        e.space();
        e.token(TokenKind::EXISTS_KW);
    }

    e.space();

    // Different object types use different fields for the name:
    // - TABLE, VIEW, INDEX, etc. use 'relation' field (RangeVar)
    // - DATABASE, SCHEMA, etc. use 'subname' field (string)
    // - COLUMN uses both 'relation' and 'subname'
    if let Some(ref relation) = n.relation {
        super::emit_range_var(e, relation);

        // For COLUMN renames, the column name is in subname
        if n.rename_type == ObjectType::ObjectColumn as i32 && !n.subname.is_empty() {
            e.space();
            e.token(TokenKind::IDENT(n.subname.clone()));
        }
    } else if !n.subname.is_empty() {
        // DATABASE, SCHEMA, etc. use subname directly
        e.token(TokenKind::IDENT(n.subname.clone()));
    }

    e.space();
    e.token(TokenKind::RENAME_KW);
    e.space();
    e.token(TokenKind::TO_KW);
    e.space();
    e.token(TokenKind::IDENT(n.newname.clone()));

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
