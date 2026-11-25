use pgls_query::protobuf::AlterTypeStmt;

use super::prelude::*;

pub(super) fn emit_alter_type_stmt(e: &mut EventEmitter, n: &AlterTypeStmt) {
    e.group_start();

    e.token(TokenKind::ALTER_KW);
    e.space();
    e.token(TokenKind::TYPE_KW);

    if !n.type_name.is_empty() {
        e.space();
        emit_dot_separated_list(e, &n.type_name);
    }

    // ALTER TYPE ... SET (option = value, ...)
    if !n.options.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::SET_KW);
        e.space();
        emit_parenthesized_list(e, &n.options, |emitter, node| {
            if let Some(NodeEnum::DefElem(def)) = node.node.as_ref() {
                // `operator_def_elem` in gram.y: the name is a `ColLabel`, so a keyword needs no quotes.
                // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L10356
                if needs_quoting_as_label(&def.defname) {
                    emit_identifier(emitter, &def.defname);
                } else {
                    emitter.token(TokenKind::IDENT(def.defname.clone()));
                }
                if let Some(ref arg) = def.arg {
                    emitter.space();
                    emitter.token(TokenKind::IDENT("=".to_string()));
                    emitter.space();
                    emit_node(emitter, arg);
                }
            }
        });
    }

    e.group_end();
}
