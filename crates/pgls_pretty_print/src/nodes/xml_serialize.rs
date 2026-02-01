use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgls_query::protobuf::{XmlOptionType, XmlSerialize};

pub(super) fn emit_xml_serialize(e: &mut EventEmitter, n: &XmlSerialize) {
    e.group_start(GroupKind::XmlSerialize);

    e.token(TokenKind::IDENT("XMLSERIALIZE".to_string()));
    e.token(TokenKind::L_PAREN);

    // XmlOptionType: XmloptionDocument=1, XmloptionContent=2
    match n.xmloption() {
        XmlOptionType::XmloptionDocument => {
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

    // INDENT option
    if n.indent {
        e.space();
        e.token(TokenKind::IDENT("INDENT".to_string()));
    }

    e.token(TokenKind::R_PAREN);

    e.group_end();
}
