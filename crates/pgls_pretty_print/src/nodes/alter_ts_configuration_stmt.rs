use super::node_list::{emit_comma_separated_list, emit_dot_separated_list};
use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};
use pgls_query::protobuf::AlterTsConfigurationStmt;

pub(super) fn emit_alter_ts_configuration_stmt(e: &mut EventEmitter, n: &AlterTsConfigurationStmt) {
    e.group_start(GroupKind::AlterTsconfigurationStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::TEXT_KW);
    e.space();
    e.token(TokenKind::SEARCH_KW);
    e.space();
    e.token(TokenKind::CONFIGURATION_KW);
    e.space();

    // Configuration name
    emit_dot_separated_list(e, &n.cfgname);

    // Kind: 0=Undefined, 1=ADD_MAPPING, 2=ALTER_MAPPING_FOR_TOKEN, 3=REPLACE_DICT, 4=REPLACE_DICT_FOR_TOKEN, 5=DROP_MAPPING
    match n.kind {
        1 => {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::ADD_KW);
            e.space();
            e.token(TokenKind::MAPPING_KW);
        }
        2 | 4 => {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::ALTER_KW);
            e.space();
            e.token(TokenKind::MAPPING_KW);
        }
        3 => {
            // REPLACE dict (without MAPPING keyword)
            // Handled below with replace flag
        }
        5 => {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::DROP_KW);
            e.space();
            e.token(TokenKind::MAPPING_KW);
        }
        _ => {}
    }

    // FOR token type
    if !n.tokentype.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::FOR_KW);
        e.space();
        emit_comma_separated_list(e, &n.tokentype, super::emit_node);
    }

    // WITH dictionaries
    if !n.dicts.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::WITH_KW);
        e.space();
        emit_comma_separated_list(e, &n.dicts, super::emit_node);
    }

    // REPLACE flag (for ALTER MAPPING ... REPLACE)
    if n.replace && (n.kind == 2 || n.kind == 4) {
        e.space();
        e.token(TokenKind::REPLACE_KW);
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
