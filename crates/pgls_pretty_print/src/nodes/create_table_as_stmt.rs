use pgls_query::{NodeEnum, protobuf::CreateTableAsStmt};

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

    // ObjectType: ObjectTable=42, ObjectMatview=24
    let is_matview = n.objtype == 24;

    if is_matview {
        e.token(TokenKind::MATERIALIZED_KW);
        e.space();
        e.token(TokenKind::VIEW_KW);
    } else {
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
    }

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

        // USING access_method
        if !into.access_method.is_empty() {
            e.space();
            e.token(TokenKind::USING_KW);
            e.space();
            e.token(TokenKind::IDENT(into.access_method.clone()));
        }

        // WITH (options) - like fillfactor
        if !into.options.is_empty() {
            e.space();
            e.token(TokenKind::WITH_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            emit_comma_separated_list(e, &into.options, |node, emitter| {
                if let Some(NodeEnum::DefElem(def)) = node.node.as_ref() {
                    emitter.token(TokenKind::IDENT(def.defname.to_lowercase()));
                    if let Some(ref arg) = def.arg {
                        emitter.space();
                        emitter.token(TokenKind::IDENT("=".to_string()));
                        emitter.space();
                        emit_node(arg, emitter);
                    }
                }
            });
            e.token(TokenKind::R_PAREN);
        }

        // ON COMMIT
        // OnCommitAction: 0=Undefined, 1=Noop (default), 2=PreserveRows, 3=DeleteRows, 4=Drop
        match into.on_commit {
            2 => {
                // OncommitPreserveRows - not commonly emitted but valid
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::ON_KW);
                e.space();
                e.token(TokenKind::COMMIT_KW);
                e.space();
                e.token(TokenKind::IDENT("preserve".to_string()));
                e.space();
                e.token(TokenKind::ROWS_KW);
            }
            3 => {
                // OncommitDeleteRows
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::ON_KW);
                e.space();
                e.token(TokenKind::COMMIT_KW);
                e.space();
                e.token(TokenKind::DELETE_KW);
                e.space();
                e.token(TokenKind::ROWS_KW);
            }
            4 => {
                // OncommitDrop
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::ON_KW);
                e.space();
                e.token(TokenKind::COMMIT_KW);
                e.space();
                e.token(TokenKind::DROP_KW);
            }
            _ => {
                // 0=Undefined, 1=Noop - no action needed
            }
        }

        // TABLESPACE
        if !into.table_space_name.is_empty() {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::TABLESPACE_KW);
            e.space();
            e.token(TokenKind::IDENT(into.table_space_name.clone()));
        }
    }

    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::AS_KW);
    e.indent_start();
    e.line(LineType::SoftOrSpace);

    if let Some(ref query) = n.query {
        if let Some(ref inner) = query.node {
            match inner {
                NodeEnum::SelectStmt(stmt) => emit_select_stmt_no_semicolon(e, stmt),
                NodeEnum::ExecuteStmt(stmt) => super::emit_execute_stmt_no_semicolon(e, stmt),
                _ => emit_node(query, e),
            }
        }
    }

    e.indent_end();

    // WITH DATA / WITH NO DATA (applies to both CREATE TABLE AS and CREATE MATERIALIZED VIEW)
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

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
