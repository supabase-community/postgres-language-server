use super::node_list::{emit_comma_separated_list, emit_dot_separated_list};
use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgt_query::protobuf::AlterTsConfigurationStmt;

pub(super) fn emit_alter_ts_configuration_stmt(e: &mut EventEmitter, n: &AlterTsConfigurationStmt) {
    e.group_start(GroupKind::AlterTsconfigurationStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::IDENT("TEXT".to_string()));
    e.space();
    e.token(TokenKind::IDENT("SEARCH".to_string()));
    e.space();
    e.token(TokenKind::IDENT("CONFIGURATION".to_string()));
    e.space();

    // Configuration name
    emit_dot_separated_list(e, &n.cfgname);

    e.space();

    // Kind: 0=Undefined, 1=ADD_MAPPING, 2=ALTER_MAPPING_FOR_TOKEN, 3=REPLACE_DICT, 4=REPLACE_DICT_FOR_TOKEN, 5=DROP_MAPPING
    match n.kind {
        1 => {
            e.token(TokenKind::IDENT("ADD".to_string()));
            e.space();
            e.token(TokenKind::IDENT("MAPPING".to_string()));
        }
        2 | 4 => {
            e.token(TokenKind::ALTER_KW);
            e.space();
            e.token(TokenKind::IDENT("MAPPING".to_string()));
        }
        3 => {
            // REPLACE dict (without MAPPING keyword)
            // Handled below with replace flag
        }
        5 => {
            e.token(TokenKind::DROP_KW);
            e.space();
            e.token(TokenKind::IDENT("MAPPING".to_string()));
        }
        _ => {}
    }

    // FOR token type
    if !n.tokentype.is_empty() {
        e.space();
        e.token(TokenKind::FOR_KW);
        e.space();
        emit_comma_separated_list(e, &n.tokentype, super::emit_node);
    }

    // WITH dictionaries
    if !n.dicts.is_empty() {
        e.space();
        e.token(TokenKind::WITH_KW);
        e.space();
        emit_comma_separated_list(e, &n.dicts, super::emit_node);
    }

    // REPLACE flag (for ALTER MAPPING ... REPLACE)
    if n.replace && (n.kind == 2 || n.kind == 4) {
        e.space();
        e.token(TokenKind::IDENT("REPLACE".to_string()));
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
