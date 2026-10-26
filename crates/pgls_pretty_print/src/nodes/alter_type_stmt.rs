use pgls_query::protobuf::AlterTypeStmt;

use super::prelude::*;

pub(super) fn emit_alter_type_stmt(e: &mut EventEmitter, n: &AlterTypeStmt) {
    e.group_start(GroupKind::AlterTypeStmt);

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
        emit_parenthesized_list(e, &n.options, |node, emitter| {
            if let Some(NodeEnum::DefElem(def)) = node.node.as_ref() {
                emitter.token(TokenKind::IDENT(def.defname.clone()));
                if let Some(ref arg) = def.arg {
                    emitter.space();
                    emitter.token(TokenKind::IDENT("=".to_string()));
                    emitter.space();
                    emit_node(arg, emitter);
                }
            }
        });
    }

    e.group_end();
}
