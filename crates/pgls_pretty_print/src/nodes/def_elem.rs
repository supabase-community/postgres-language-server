use pgls_query::protobuf::DefElem;

use super::prelude::*;

pub(super) fn emit_def_elem(e: &mut EventEmitter, n: &DefElem) {
    e.group_start();

    // `namespace.name`: both parts are any word, keywords included.
    // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L6529
    // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L3082
    if !n.defnamespace.is_empty() {
        emit_label_maybe_quoted(e, &n.defnamespace);
        e.token(TokenKind::DOT);
    }
    emit_label_maybe_quoted(e, &n.defname);

    // Emit the option value if present
    if let Some(ref arg) = n.arg {
        e.space();
        e.token(TokenKind::IDENT("=".to_string()));
        e.space();

        // String values in DefElem should be quoted as string literals
        if let Some(node_enum) = &arg.node {
            match node_enum {
                NodeEnum::String(s) => {
                    emit_string_literal(e, s);
                }
                _ => {
                    emit_node(e, arg);
                }
            }
        }
    }

    e.group_end();
}
