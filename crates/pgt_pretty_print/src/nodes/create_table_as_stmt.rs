use pgt_query::protobuf::CreateTableAsStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};

use super::emit_node;

pub(super) fn emit_create_table_as_stmt(e: &mut EventEmitter, n: &CreateTableAsStmt) {
    e.group_start(GroupKind::CreateTableAsStmt);

    e.token(TokenKind::CREATE_KW);
    e.space();

    // ObjectType: 0=TABLE, 1=MATVIEW
    if n.objtype == 1 {
        e.token(TokenKind::MATERIALIZED_KW);
        e.space();
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
        }
    }

    e.space();
    e.token(TokenKind::AS_KW);
    e.indent_start();
    e.line(LineType::SoftOrSpace);

    if let Some(ref query) = n.query {
        emit_node(query, e);
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
