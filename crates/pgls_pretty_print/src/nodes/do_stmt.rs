use pgls_query::{NodeEnum, protobuf::DoStmt};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};

use super::string::{
    DollarQuoteHint, emit_dollar_quoted_str_with_hint, emit_identifier_maybe_quoted, emit_keyword,
};

pub(super) fn emit_do_stmt(e: &mut EventEmitter, n: &DoStmt) {
    e.group_start(GroupKind::DoStmt);

    e.token(TokenKind::DO_KW);

    let mut language: Option<(String, i32)> = None;
    let mut body: Option<(String, i32)> = None;

    for arg in &n.args {
        match &arg.node {
            Some(NodeEnum::DefElem(def_elem)) => match def_elem.defname.as_str() {
                "language" => {
                    if let Some(lang_node) = &def_elem.arg {
                        if let Some(NodeEnum::String(s)) = &lang_node.node {
                            language = Some((s.sval.clone(), def_elem.location));
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
                            body = Some((s.sval.clone(), def_elem.location));
                        } else {
                            debug_assert!(false, "DoStmt AS def_elem should hold a String node");
                        }
                    } else {
                        debug_assert!(false, "DoStmt AS def_elem is missing arg");
                    }
                }
                other => {
                    debug_assert!(false, "Unexpected defname '{other}' in DoStmt args");
                }
            },
            unexpected => {
                debug_assert!(unexpected.is_none(), "Unexpected node type in DoStmt args");
            }
        }
    }

    // Determine order based on location - smaller location comes first
    // This preserves the original SQL order (LANGUAGE before body or body before LANGUAGE)
    let language_first = match (&language, &body) {
        (Some((_, lang_loc)), Some((_, body_loc))) => lang_loc < body_loc,
        (Some(_), None) => true,
        (None, Some(_)) => false,
        (None, None) => false,
    };

    if language_first {
        if let Some((lang, _)) = &language {
            e.line(LineType::SoftOrSpace);
            emit_keyword(e, "LANGUAGE");
            e.space();
            emit_identifier_maybe_quoted(e, lang);
        }
        if let Some((code, _)) = &body {
            e.line(LineType::SoftOrSpace);
            emit_dollar_quoted_str_with_hint(e, code, DollarQuoteHint::Do);
        }
    } else {
        if let Some((code, _)) = &body {
            e.line(LineType::SoftOrSpace);
            emit_dollar_quoted_str_with_hint(e, code, DollarQuoteHint::Do);
        }
        if let Some((lang, _)) = &language {
            e.line(LineType::SoftOrSpace);
            emit_keyword(e, "LANGUAGE");
            e.space();
            emit_identifier_maybe_quoted(e, lang);
        }
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}
