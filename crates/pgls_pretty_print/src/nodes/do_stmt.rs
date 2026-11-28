use pgls_query::protobuf::DoStmt;

use super::prelude::*;

pub(super) fn emit_do_stmt(e: &mut EventEmitter, n: &DoStmt) {
    e.group_start();

    e.token(TokenKind::DO_KW);

    // The parser keeps the items in the order they were written, and accepts any number of them;
    // only executing the statement rejects a second body or language.
    // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L9154
    for arg in &n.args {
        let Some(NodeEnum::DefElem(def_elem)) = &arg.node else {
            debug_assert!(false, "Unexpected node type in DoStmt args");
            continue;
        };
        let Some(NodeEnum::String(value)) = def_elem.arg.as_ref().and_then(|arg| arg.node.as_ref())
        else {
            debug_assert!(false, "DoStmt def_elem should hold a String node");
            continue;
        };
        match def_elem.defname.as_str() {
            "language" => {
                e.line(LineType::SoftOrSpace);
                emit_keyword(e, "LANGUAGE");
                e.space();
                emit_word_or_string(e, &value.sval);
            }
            "as" => {
                e.line(LineType::SoftOrSpace);
                emit_body_literal(e, &value.sval);
            }
            other => {
                debug_assert!(false, "Unexpected defname '{other}' in DoStmt args");
            }
        }
    }

    e.group_end();
}
