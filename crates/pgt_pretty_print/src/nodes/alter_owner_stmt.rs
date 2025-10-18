use pgt_query::protobuf::AlterOwnerStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_alter_owner_stmt(e: &mut EventEmitter, n: &AlterOwnerStmt) {
    e.group_start(GroupKind::AlterOwnerStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();

    // Object type - map object_type enum to SQL keyword
    // Based on ObjectType enum in protobuf.rs
    match n.object_type {
        1 => {
            // ObjectAccessMethod
            e.token(TokenKind::IDENT("ACCESS".to_string()));
            e.space();
            e.token(TokenKind::IDENT("METHOD".to_string()));
        }
        2 => e.token(TokenKind::IDENT("AGGREGATE".to_string())),
        8 => e.token(TokenKind::IDENT("COLLATION".to_string())),
        9 => e.token(TokenKind::IDENT("CONVERSION".to_string())),
        10 => e.token(TokenKind::DATABASE_KW),
        13 => e.token(TokenKind::DOMAIN_KW),
        15 => {
            // ObjectEventTrigger
            e.token(TokenKind::IDENT("EVENT".to_string()));
            e.space();
            e.token(TokenKind::IDENT("TRIGGER".to_string()));
        }
        17 => {
            // ObjectFdw
            e.token(TokenKind::IDENT("FOREIGN".to_string()));
            e.space();
            e.token(TokenKind::IDENT("DATA".to_string()));
            e.space();
            e.token(TokenKind::IDENT("WRAPPER".to_string()));
        }
        18 => {
            // ObjectForeignServer
            e.token(TokenKind::IDENT("SERVER".to_string()));
        }
        19 => {
            // ObjectForeignTable
            e.token(TokenKind::IDENT("FOREIGN".to_string()));
            e.space();
            e.token(TokenKind::TABLE_KW);
        }
        20 => e.token(TokenKind::FUNCTION_KW),
        22 => e.token(TokenKind::IDENT("LANGUAGE".to_string())),
        23 => {
            // ObjectLargeobject
            e.token(TokenKind::IDENT("LARGE".to_string()));
            e.space();
            e.token(TokenKind::IDENT("OBJECT".to_string()));
        }
        24 => {
            // ObjectMatview
            e.token(TokenKind::IDENT("MATERIALIZED".to_string()));
            e.space();
            e.token(TokenKind::VIEW_KW);
        }
        25 => {
            // ObjectOpclass
            e.token(TokenKind::IDENT("OPERATOR".to_string()));
            e.space();
            e.token(TokenKind::IDENT("CLASS".to_string()));
        }
        26 => e.token(TokenKind::IDENT("OPERATOR".to_string())),
        27 => {
            // ObjectOpfamily
            e.token(TokenKind::IDENT("OPERATOR".to_string()));
            e.space();
            e.token(TokenKind::IDENT("FAMILY".to_string()));
        }
        30 => e.token(TokenKind::IDENT("PROCEDURE".to_string())),
        31 => e.token(TokenKind::IDENT("PUBLICATION".to_string())),
        35 => e.token(TokenKind::IDENT("ROUTINE".to_string())),
        37 => e.token(TokenKind::SCHEMA_KW),
        38 => e.token(TokenKind::SEQUENCE_KW),
        39 => e.token(TokenKind::IDENT("SUBSCRIPTION".to_string())),
        40 => {
            // ObjectStatisticExt
            e.token(TokenKind::IDENT("STATISTICS".to_string()));
        }
        42 => e.token(TokenKind::TABLE_KW),
        43 => e.token(TokenKind::IDENT("TABLESPACE".to_string())),
        46 => {
            // ObjectTsconfiguration
            e.token(TokenKind::IDENT("TEXT".to_string()));
            e.space();
            e.token(TokenKind::IDENT("SEARCH".to_string()));
            e.space();
            e.token(TokenKind::IDENT("CONFIGURATION".to_string()));
        }
        47 => {
            // ObjectTsdictionary
            e.token(TokenKind::IDENT("TEXT".to_string()));
            e.space();
            e.token(TokenKind::IDENT("SEARCH".to_string()));
            e.space();
            e.token(TokenKind::IDENT("DICTIONARY".to_string()));
        }
        50 => e.token(TokenKind::TYPE_KW),
        52 => e.token(TokenKind::VIEW_KW),
        _ => e.token(TokenKind::IDENT("OBJECT".to_string())), // Fallback for unsupported types
    }

    e.space();

    // Object name (could be qualified name or simple identifier)
    if let Some(ref obj) = n.object {
        super::emit_node(obj, e);
    }

    // OWNER TO
    e.space();
    e.token(TokenKind::IDENT("OWNER".to_string()));
    e.space();
    e.token(TokenKind::TO_KW);

    // New owner
    if let Some(ref newowner) = n.newowner {
        e.space();
        super::emit_role_spec(e, newowner);
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}
