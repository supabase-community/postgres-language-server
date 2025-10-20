use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgt_query::protobuf::XmlSerialize;

pub(super) fn emit_xml_serialize(e: &mut EventEmitter, n: &XmlSerialize) {
    e.group_start(GroupKind::XmlSerialize);

    e.token(TokenKind::IDENT("XMLSERIALIZE".to_string()));
    e.token(TokenKind::L_PAREN);

    // xmloption: DOCUMENT or CONTENT (0 = content, 1 = document)
    match n.xmloption {
        1 => {
            e.token(TokenKind::IDENT("DOCUMENT".to_string()));
            e.space();
        }
        _ => {
            e.token(TokenKind::IDENT("CONTENT".to_string()));
            e.space();
        }
    }

    // Expression to serialize
    if let Some(ref expr) = n.expr {
        super::emit_node(expr, e);
    }

    // AS type
    if let Some(ref type_name) = n.type_name {
        e.space();
        e.token(TokenKind::AS_KW);
        e.space();
        super::emit_type_name(e, type_name);
    }

    e.token(TokenKind::R_PAREN);

    e.group_end();
}
