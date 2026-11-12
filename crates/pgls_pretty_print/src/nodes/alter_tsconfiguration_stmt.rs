use pgls_query::protobuf::{AlterTsConfigType, AlterTsConfigurationStmt};

use super::prelude::*;

pub(super) fn emit_alter_tsconfiguration_stmt(e: &mut EventEmitter, n: &AlterTsConfigurationStmt) {
    e.group_start();

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

    let kind = n.kind();

    e.line(LineType::SoftOrSpace);
    match kind {
        AlterTsConfigType::AlterTsconfigAddMapping => e.token(TokenKind::ADD_KW),
        AlterTsConfigType::AlterTsconfigDropMapping => e.token(TokenKind::DROP_KW),
        _ => e.token(TokenKind::ALTER_KW),
    }
    e.space();
    e.token(TokenKind::MAPPING_KW);

    if kind == AlterTsConfigType::AlterTsconfigDropMapping && n.missing_ok {
        e.space();
        emit_if_exists(e);
    }

    // FOR token types (every form except the token-less REPLACE)
    if !n.tokentype.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::FOR_KW);
        e.space();
        emit_comma_separated_list(e, &n.tokentype, emit_node);
    }

    if n.replace {
        // REPLACE old_dict WITH new_dict: dicts holds exactly the old and the new dictionary
        if let [old, new] = n.dicts.as_slice() {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::REPLACE_KW);
            e.space();
            emit_dictionary(e, old);
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::WITH_KW);
            e.space();
            emit_dictionary(e, new);
        }
    } else if !n.dicts.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::WITH_KW);
        e.space();
        emit_comma_separated_list(e, &n.dicts, emit_dictionary);
    }

    e.group_end();
}

/// A dictionary is a possibly qualified name, stored as a `List` of its parts.
fn emit_dictionary(e: &mut EventEmitter, node: &Node) {
    match node.node.as_ref() {
        Some(NodeEnum::List(list)) => emit_dot_separated_list(e, &list.items),
        _ => emit_node(e, node),
    }
}
