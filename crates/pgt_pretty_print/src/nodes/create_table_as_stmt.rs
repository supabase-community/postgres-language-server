use pgt_query::{NodeEnum, protobuf::CreateTableAsStmt};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};

use super::{
    emit_node, node_list::emit_comma_separated_list, select_stmt::emit_select_stmt_no_semicolon,
    string::emit_string,
};

pub(super) fn emit_create_table_as_stmt(e: &mut EventEmitter, n: &CreateTableAsStmt) {
    e.group_start(GroupKind::CreateTableAsStmt);

    e.token(TokenKind::CREATE_KW);
    e.space();

    // ObjectType: 0=TABLE, 1=MATVIEW
    if n.objtype == 1 {
        e.token(TokenKind::MATERIALIZED_KW);
        e.space();
    }

    if let Some(ref into) = n.into {
        if let Some(ref rel) = into.rel {
            match rel.relpersistence.as_str() {
                "t" => {
                    e.token(TokenKind::TEMPORARY_KW);
                    e.space();
                }
                "u" => {
                    e.token(TokenKind::UNLOGGED_KW);
                    e.space();
                }
                _ => {}
            }
        }
    }

    e.token(TokenKind::TABLE_KW);

    if n.if_not_exists {
        e.space();
        e.token(TokenKind::IF_KW);
        e.space();
        e.token(TokenKind::NOT_KW);
        e.space();
        e.token(TokenKind::EXISTS_KW);
    }

    e.space();

    if let Some(ref into) = n.into {
        if let Some(ref rel) = into.rel {
            super::emit_range_var(e, rel);

            if !into.col_names.is_empty() {
                e.space();
                e.token(TokenKind::L_PAREN);
                emit_comma_separated_list(e, &into.col_names, |node, e| {
                    let string = assert_node_variant!(String, node);
                    emit_string(e, string);
                });
                e.token(TokenKind::R_PAREN);
            }
        }
    }

    e.space();
    e.token(TokenKind::AS_KW);
    e.indent_start();
    e.line(LineType::SoftOrSpace);

    if let Some(ref query) = n.query {
        if let Some(ref inner) = query.node {
            match inner {
                NodeEnum::SelectStmt(stmt) => emit_select_stmt_no_semicolon(e, stmt),
                _ => emit_node(query, e),
            }
        }
    }

    e.indent_end();

    // WITH DATA / WITH NO DATA
    if n.objtype == 1 {
        // Materialized view
        if let Some(ref into) = n.into {
            if into.skip_data {
                e.space();
                e.token(TokenKind::WITH_KW);
                e.space();
                e.token(TokenKind::NO_KW);
                e.space();
                e.token(TokenKind::DATA_KW);
            }
        }
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
