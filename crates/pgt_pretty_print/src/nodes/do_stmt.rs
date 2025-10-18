use pgt_query::{NodeEnum, protobuf::DoStmt};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

use super::string::{emit_dollar_quoted_str, emit_identifier_maybe_quoted, emit_keyword};

pub(super) fn emit_do_stmt(e: &mut EventEmitter, n: &DoStmt) {
    e.group_start(GroupKind::DoStmt);

    e.token(TokenKind::DO_KW);

    let mut language: Option<String> = None;
    let mut body: Option<String> = None;

    for arg in &n.args {
        match &arg.node {
            Some(NodeEnum::DefElem(def_elem)) => match def_elem.defname.as_str() {
                "language" => {
                    if let Some(lang_node) = &def_elem.arg {
                        if let Some(NodeEnum::String(s)) = &lang_node.node {
                            language = Some(s.sval.clone());
                        } else {
                            debug_assert!(
                                false,
                                "DoStmt language def_elem should hold a String node"
                            );
                        }
                    } else {
                        debug_assert!(false, "DoStmt language def_elem is missing arg");
                    }
                }
                "as" => {
                    if let Some(code_node) = &def_elem.arg {
                        if let Some(NodeEnum::String(s)) = &code_node.node {
                            body = Some(s.sval.clone());
                        } else {
                            debug_assert!(false, "DoStmt AS def_elem should hold a String node");
                        }
                    } else {
                        debug_assert!(false, "DoStmt AS def_elem is missing arg");
                    }
                }
                other => {
                    debug_assert!(false, "Unexpected defname '{}' in DoStmt args", other);
                }
            },
            unexpected => {
                debug_assert!(unexpected.is_none(), "Unexpected node type in DoStmt args");
            }
        }
    }

    if let Some(lang) = language {
        e.space();
        emit_keyword(e, "LANGUAGE");
        e.space();
        emit_identifier_maybe_quoted(e, &lang);
    }

    if let Some(code) = body {
        e.space();
        emit_dollar_quoted_str(e, &code);
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}
